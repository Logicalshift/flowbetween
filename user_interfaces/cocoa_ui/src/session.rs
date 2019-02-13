use super::event::*;
use super::cocoa_ui::*;
use super::property::*;
use super::view_canvas::*;
use super::core_graphics_ffi::*;

use flo_ui::*;
use flo_stream::*;
use flo_canvas::*;
use flo_cocoa_pipe::*;

use futures::*;
use futures::executor;
use futures::executor::Spawn;

use cocoa::base::{id, nil};
use cocoa::foundation::NSString;
use objc::rc::*;
use objc::runtime::*;

use std::sync::*;
use std::collections::HashMap;

#[link(name = "Foundation", kind = "framework")]
extern {
    pub static NSDefaultRunLoopMode: id;
    pub static NSModalPanelRunLoopMode: id;
    pub static NSEventTrackingRunLoopMode: id;
}

///
/// Basis class for a Cocoa session
///
pub struct CocoaSession {
    /// The ID of this session
    session_id: usize,

    /// Reference to the FloControl used to interface between the stream and the Objective-C/Swift side of the application
    target_object: StrongPtr,

    /// Maps IDs to windows
    windows: HashMap<usize, StrongPtr>,

    /// Maps IDs to views
    views: HashMap<usize, StrongPtr>,

    /// Maps view IDs to event objects
    view_events: HashMap<usize, StrongPtr>,

    /// Maps IDs to viewmodels
    viewmodels: HashMap<usize, StrongPtr>,

    /// Maps view IDs to their canvas states
    canvases: HashMap<usize, ViewCanvas>,

    /// Publisher where we send the actions to
    action_publisher: Publisher<Vec<AppAction>>,

    /// The stream of actions for this session (or None if we aren't monitoring for actions)
    actions: Option<Spawn<Subscriber<Vec<AppAction>>>>,

    /// The event publisher for this session
    events: Publisher<Vec<AppEvent>>
}

///
/// Object to notify when it's time to drain the action stream again
///
struct CocoaSessionNotify {
    notify_object: Mutex<NotifyRef>
}

///
/// Reference to an object to be notified
///
struct NotifyRef {
    target_object: WeakPtr
}

///
/// Retains an objective-C strong ptr
///
#[inline] unsafe fn retain(obj: &StrongPtr) -> *mut Object {
    let obj = **obj;
    msg_send!(obj, retain);
    obj
}

impl CocoaSession {
    ///
    /// Creates a new CocoaSession
    ///
    pub fn new(obj: &StrongPtr, session_id: usize) -> CocoaSession {
        CocoaSession {
            session_id:         session_id,
            target_object:      obj.clone(),
            windows:            HashMap::new(),
            views:              HashMap::new(),
            view_events:        HashMap::new(),
            viewmodels:         HashMap::new(),
            canvases:           HashMap::new(),
            actions:            None,
            action_publisher:   Publisher::new(1),
            events:             Publisher::new(20)
        }
    }

    ///
    /// Creates a user interface implementation for this session
    ///
    pub fn create_user_interface(&mut self) -> impl UserInterface<Vec<AppAction>, Vec<AppEvent>, ()> {
        // Start listening for actions if we aren't already, by spawning a subscriber to our publisher
        if self.actions.is_none() {
            self.actions = Some(executor::spawn(self.action_publisher.subscribe()));
            self.start_listening();
        }

        // Create the subscriber to receive events sent from the user interface
        let action_publisher    = self.action_publisher.republish();
        let events              = self.events.republish();

        // Generate a cocoa user interface
        CocoaUserInterface::new(action_publisher, events)
    }

    ///
    /// Listens for actions from the specified stream
    ///
    fn start_listening(&mut self) {
        unsafe {
            autoreleasepool(|| {
                // Wake up the object on the main thread
                msg_send!(*self.target_object, performSelectorOnMainThread: sel!(actionStreamReady) withObject: nil waitUntilDone: NO);
            });
        }
    }

    ///
    /// Drains any pending messages from the actions stream
    ///
    pub fn drain_action_stream(&mut self) {
        autoreleasepool(move || {
            // Create the object to notify when there's an update
            let notify = Arc::new(CocoaSessionNotify::new(&self.target_object));

            // Drain the stream until it's empty or it blocks
            loop {
                let next = self.actions
                    .as_mut()
                    .map(|actions| actions.poll_stream_notify(&notify, 0))
                    .unwrap_or_else(|| Ok(Async::NotReady));

                match next {
                    Ok(Async::NotReady)     => { break; }
                    Ok(Async::Ready(None))  => {
                        // Session has finished
                        break;
                    }

                    Ok(Async::Ready(Some(actions))) => {
                        for action in actions {
                            // Perform the action
                            self.dispatch_app_action(action);
                        }
                    }

                    Err(_) => {
                        // Action stream should never produce any errors
                        unimplemented!("Action stream should never produce any errors")
                    }
                }
            }
        });
    }

    ///
    /// Performs an application action on this object
    ///
    pub fn dispatch_app_action(&mut self, action: AppAction) {
        use self::AppAction::*;

        match action {
            CreateWindow(window_id)             => { self.create_window(window_id); }
            Window(window_id, window_action)    => { self.windows.get(&window_id).map(|window| self.dispatch_window_action(window, window_action)); }
            
            CreateView(view_id, view_type)      => { self.create_view(view_id, view_type); },
            DeleteView(view_id)                 => { self.delete_view(view_id); }
            View(view_id, view_action)          => { self.dispatch_view_action(view_id, view_action); }

            CreateViewModel(viewmodel_id)       => { self.create_viewmodel(viewmodel_id); },
            DeleteViewModel(viewmodel_id)       => { self.viewmodels.remove(&viewmodel_id); },
            ViewModel(view_model_id, action)    => { self.viewmodels.get(&view_model_id).map(|viewmodel| self.dispatch_viewmodel_action(viewmodel, action)); }
        }
    }

    ///
    /// Creates a new window and assigns the specified ID to it
    ///
    fn create_window(&mut self, new_window_id: usize) {
        unsafe {
            // Fetch the window class to create
            let window_class = (**self.target_object).get_ivar::<*mut Class>("_windowClass");

            // Allocate and initialise it
            let window: *mut Object = msg_send!(*window_class, alloc);
            let window = msg_send!(window, init: *self.target_object);
            let window = StrongPtr::new(window);

            // Immediately request a tick from the new window (this is in case one was queued before the window was created)
            msg_send!((*window), requestTick);

            // Store it away
            self.windows.insert(new_window_id, window);
        }
    }

    ///
    /// Dispatches an action to the specified window
    ///
    fn dispatch_window_action(&self, window: &StrongPtr, action: WindowAction) {
        use self::WindowAction::*;

        unsafe {
            match action {
                RequestTick             => { msg_send!((**window), requestTick); }
                Open                    => { msg_send!((**window), windowOpen); }
                SetRootView(view_id)    => { self.views.get(&view_id).map(|view| msg_send!((**window), windowSetRootView: **view)); }
            }
        }
    }

    ///
    /// Creates a new view and assigns the specified ID to it
    ///
    fn create_view(&mut self, new_view_id: usize, view_type: ViewType) {
        use self::ViewType::*;

        unsafe {
            // Fetch the view class to create
            let view_class = (**self.target_object).get_ivar::<*mut Class>("_viewClass");

            // Allocate and initialise it
            let view: *mut Object = match view_type {
                Empty           => { msg_send!(*view_class, createAsEmpty) }
                Button          => { msg_send!(*view_class, createAsButton) }
                ContainerButton => { msg_send!(*view_class, createAsContainerButton) }
                Slider          => { msg_send!(*view_class, createAsSlider) }
                TextBox         => { msg_send!(*view_class, createAsTextBox) }
                CheckBox        => { msg_send!(*view_class, createAsCheckBox) }
                Scrolling       => { msg_send!(*view_class, createAsScrolling) }
                Popup           => { msg_send!(*view_class, createAsPopup) }
            };

            let view = StrongPtr::new(view);

            // Store it away
            self.views.insert(new_view_id, view);
        }
    }

    ///
    /// Removes a view from this object
    ///
    pub fn delete_view(&mut self, old_view_id: usize) {
        self.views.remove(&old_view_id);
        self.view_events.remove(&old_view_id);
    }

    ///
    /// Retrieves the events object for a particular view
    ///
    pub fn events_for_view(&mut self, view_id: usize) -> StrongPtr {
        if let Some(events) = self.view_events.get(&view_id).cloned() {
            // Already got an events object for this view
            events
        } else {
            // Create a new events object
            let events = FloEvents::create_object(self.events.republish(), self.session_id, view_id);

            // Associate it with the view
            self.view_events.insert(view_id, events.clone());

            events
        }
    }

    ///
    /// Dispatches an action to the specified view
    ///
    fn dispatch_view_action(&mut self, view_id: usize, action: ViewAction) {
        use self::ViewAction::*;

        let views = &self.views;

        if let Some(view) = views.get(&view_id) {
            unsafe {
                match action {
                    RequestEvent(event_type, name)          => { self.request_view_event(view_id, event_type, name); }

                    RemoveFromSuperview                     => { msg_send!(**view, viewRemoveFromSuperview); }
                    AddSubView(view_id)                     => { self.views.get(&view_id).map(|subview| { msg_send!((**view), viewAddSubView: retain(subview)) }); }
                    InsertSubView(view_id, index)           => { self.views.get(&view_id).map(|subview| { msg_send!((**view), viewInsertSubView: retain(subview) atIndex: index as u32) }); }
                    SetBounds(bounds)                       => { self.set_bounds(view, bounds); }
                    SetPadding(left, top, right, bottom)    => { self.set_padding(view, left, top, right, bottom); }
                    SetZIndex(z_index)                      => { msg_send!(**view, viewSetZIndex: z_index); }
                    SetForegroundColor(col)                 => { let (r, g, b, a) = col.to_rgba_components(); msg_send!(**view, viewSetForegroundRed: r as f64 green: g as f64 blue: b as f64 alpha: a as f64); }
                    SetBackgroundColor(col)                 => { let (r, g, b, a) = col.to_rgba_components(); msg_send!(**view, viewSetBackgroundRed: r as f64 green: g as f64 blue: b as f64 alpha: a as f64); }

                    SetId(_id)                              => { /* TODO? */ }
                    SetText(property)                       => { msg_send!(**view, viewSetText: &*self.flo_property(property)); }
                    SetFontSize(size)                       => { msg_send!(**view, viewSetFontSize: size); }
                    SetFontWeight(weight)                   => { msg_send!(**view, viewSetFontWeight: weight); }
                    SetTextAlignment(align)                 => { msg_send!(**view, viewSetTextAlignment: Self::text_alignment_value(align)); }

                    SetImage(image)                         => { msg_send!(**view, viewSetImage: self.create_ns_image(image)); }
                    SetState(view_state)                    => { self.set_view_state(view, view_state); },

                    Popup(action)                           => { /* TODO */ }

                    SetScrollMinimumSize(width, height)     => { msg_send!(**view, viewSetScrollMinimumSizeWithWidth: width height: height); }
                    SetHorizontalScrollBar(visibility)      => { msg_send!(**view, viewSetHorizontalScrollVisibility: Self::scroll_visibility_value(visibility)); },
                    SetVerticalScrollBar(visibility)        => { msg_send!(**view, viewSetVerticalScrollVisibility: Self::scroll_visibility_value(visibility)); },

                    Draw(canvas_actions)                    => { 
                        let view = view.clone();
                        self.draw(view_id, &view, canvas_actions); 
                    }
                }
            }
        }
    }

    ///
    /// Updates the state of a view
    ///
    fn set_view_state(&self, view: &StrongPtr, view_state: ViewStateUpdate) {
        use self::ViewStateUpdate::*;

        unsafe {
            match view_state {
                Selected(property)          => { msg_send!(**view, viewSetSelected: &*self.flo_property(property)); },
                Badged(property)            => { msg_send!(**view, viewSetBadged: &*self.flo_property(property)); },
                Enabled(property)           => { msg_send!(**view, viewSetEnabled: &*self.flo_property(property)); },
                Value(property)             => { msg_send!(**view, viewSetValue: &*self.flo_property(property)); },
                Range(lower, upper)         => { msg_send!(**view, viewSetRangeWithLower: &*self.flo_property(lower) upper: &*self.flo_property(upper)); },
                FocusPriority(property)     => { msg_send!(**view, viewSetFocusPriority: &*self.flo_property(property)); }
            }
        }
    }

    ///
    /// Creates a view canvas for this session
    ///
    fn create_view_canvas(view: &StrongPtr) -> ViewCanvas {
        let view_src        = view.clone();

        let view            = view_src.clone();
        let clear_canvas    = move || { unsafe { msg_send!(*view, viewClearCanvas); } };
        let view            = view_src.clone();
        let copy_layer      = move |layer_id| { 
            unsafe { 
                let layer_copy: *mut Object = msg_send!(*view, viewCopyLayerWithId: layer_id);
                let layer_copy = StrongPtr::retain(layer_copy);
                layer_copy
            } 
        };
        let view            = view_src.clone();
        let restore_layer   = move |layer_id, layer_obj: StrongPtr| { unsafe { msg_send!(*view, viewRestoreLayerTo: layer_id fromCopy: *layer_obj) } };

        ViewCanvas::new(clear_canvas, copy_layer, restore_layer)
    }

    ///
    /// Performs some drawing actions on the specified view
    ///
    fn draw(&mut self, view_id: usize, view: &StrongPtr, actions: Vec<Draw>) {
        unsafe {
            // Fetch the events for redraw callbacks
            let flo_events = self.events_for_view(view_id);

            // Fetch the canvas for this view
            let canvas = self.canvases.entry(view_id).or_insert_with(|| Self::create_view_canvas(view));

            canvas.draw(actions, move |layer_id| {
                let graphics_context: CGContextRef = msg_send!(**view, viewGetCanvasForDrawing: retain(&flo_events) layer: layer_id);

                if graphics_context.is_null() {
                    None
                } else {
                    Some(CFRef::from(graphics_context.retain()))
                }
            });

            // Finished drawing
            msg_send!(**view, viewFinishedDrawing);
            msg_send!(**view, viewSetTransform: canvas.get_transform());
        }
    }

    ///
    /// Forces a canvas to be redrawn with a new size
    ///
    pub fn redraw_canvas_for_view(&mut self, view_id: usize, size: CGSize, bounds: CGRect) {
        unsafe {
            // Fetch the events for redraw callbacks
            let flo_events = self.events_for_view(view_id);

            // Fetch the canvas for this view
            let view    = self.views.get(&view_id);
            let canvas  = self.canvases.entry(view_id).or_insert_with(|| Self::create_view_canvas(view.unwrap()));

            if let Some(view) = view {
                // Perform the drawing actions on the canvas
                canvas.set_viewport(size, bounds);
                canvas.redraw(move |layer_id| {
                    let graphics_context: CGContextRef = msg_send!(**view, viewGetCanvasForDrawing: retain(&flo_events) layer: layer_id);

                    if graphics_context.is_null() {
                        None
                    } else {
                        Some(CFRef::from(graphics_context.retain()))
                    }
                });

                // Finished drawing
                msg_send!(**view, viewFinishedDrawing);
                msg_send!(**view, viewSetTransform: canvas.get_transform());
            }
        }
    }

    ///
    /// Sends a tick event
    ///
    pub fn tick(&mut self) {
        // Create a place to send the tick to
        let mut events = executor::spawn(self.events.republish());

        // Send a tick event
        events.wait_send(vec![AppEvent::Tick]).ok();
    }

    ///
    /// Requests an event for a particular view with the specified name
    ///
    fn request_view_event(&mut self, view_id: usize, event_type: ViewEvent, name: String) {
        unsafe {
            use self::ViewEvent::*;

            let flo_events  = self.events_for_view(view_id);
            let views       = &self.views;
            let name        = NSString::alloc(nil).init_str(&name);
            let name        = StrongPtr::new(name);

            if let Some(view) = views.get(&view_id) {
                match event_type {
                    Click                           => { msg_send!(**view, requestClick: retain(&flo_events) withName: retain(&name)); }
                    Dismiss                         => { msg_send!(**view, requestDismiss: retain(&flo_events) withName: retain(&name)); }
                    VirtualScroll(width, height)    => { msg_send!(**view, requestVirtualScroll: retain(&flo_events) withName: retain(&name) width: width as f64 height: height as f64); }
                    Paint(device)                   => { msg_send!(**view, requestPaintWithDeviceId: device as u32 events: retain(&flo_events) withName: retain(&name)); }
                    Drag                            => { msg_send!(**view, requestDrag: retain(&flo_events) withName: retain(&name)); }
                    Focused                         => { msg_send!(**view, requestFocused: retain(&flo_events) withName: retain(&name)); }
                    EditValue                       => { msg_send!(**view, requestEditValue: retain(&flo_events) withName: retain(&name)); }
                    SetValue                        => { msg_send!(**view, requestSetValue: retain(&flo_events) withName: retain(&name)); }
                    CancelEdit                      => { msg_send!(**view, requestCancelEdit: retain(&flo_events) withName: retain(&name)); }
                }
            }
        }
    }

    ///
    /// Creates some glib bytes from an image data object
    /// 
    fn bytes_from_image_data(image_data: &dyn ImageData) -> id {
        unsafe {
            // Read the image data out into a byte buffer
            let mut data = vec![];
            image_data.read()
                .read_to_end(&mut data)
                .unwrap();

            // Turn into a NSData object
            let data_obj: id = msg_send!(class!(NSData), alloc);
            let data_obj: id = msg_send!(data_obj, initWithBytes: data.as_ptr() length: data.len());

            data_obj
        }
    }

    ///
    /// Creates an NSImage from an image resource
    ///
    fn create_ns_image(&self, image: Resource<Image>) -> id {
        use self::Image::*;

        unsafe {
            // Create the NSData for the image
            let image_data = match &*image {
                &Png(ref image_data) => Self::bytes_from_image_data(&**image_data),
                &Svg(ref image_data) => Self::bytes_from_image_data(&**image_data)
            };

            // Load into an image
            let image: id = msg_send!(class!(NSImage), alloc);
            let image: id = msg_send!(image, initWithData: image_data);

            image
        }
    }

    ///
    /// Returns the integer value equivalent to a text alignment
    ///
    fn text_alignment_value(align: TextAlign) -> u32 {
        use self::TextAlign::*;

        match align {
            Left    => 0,
            Center  => 1,
            Right   => 2
        }
    }

    ///
    /// Returns the integer value equivalent to a scroll bar visibility
    ///
    fn scroll_visibility_value(visibility: ScrollBarVisibility) -> u32 {
        use self::ScrollBarVisibility::*;

        match visibility {
            Never           => 0,
            Always          => 1,
            OnlyIfNeeded    => 2
        }
    }

    ///
    /// Sends a request to a view to set its bounding box
    ///
    fn set_bounds(&self, view: &StrongPtr, bounds: AppBounds) {
        self.set_position(view, 0, bounds.x1);
        self.set_position(view, 1, bounds.y1);
        self.set_position(view, 2, bounds.x2);
        self.set_position(view, 3, bounds.y2);
    }

    ///
    /// Sends a request to a view to set its bounding box
    ///
    fn set_padding(&self, view: &StrongPtr, left: f64, top: f64, right: f64, bottom: f64) {
        unsafe {
            msg_send!(**view, viewSetPaddingWithLeft: left top: top right: right bottom: bottom);
        }
    }

    ///
    /// Sets a request to set the position of a side of a view
    ///
    fn set_position(&self, view: &StrongPtr, side: i32, pos: AppPosition) {
        use self::AppPosition::*;

        unsafe {
            match pos {
                At(pos)                     => { msg_send!(**view, viewSetSide: side at: pos) },
                Floating(prop, offset)      => {
                    let floating_property = self.flo_property(prop);
                    msg_send!(**view, viewSetSide: side offset: offset floating: floating_property) 
                },
                Offset(offset)              => { msg_send!(**view, viewSetSide: side offset: offset); },
                Stretch(amount)             => { msg_send!(**view, viewSetSide: side stretch: amount); },
                Start                       => { msg_send!(**view, viewSetSideAtStart: side); },
                End                         => { msg_send!(**view, viewSetSideAtEnd: side); },
                After                       => { msg_send!(**view, viewSetSideAfter: side); }
            }
        }
    }

    ///
    /// Creates a new viewmodel with the specified ID
    ///
    fn create_viewmodel(&mut self, viewmodel_id: usize) {
        unsafe {
            // Create the viewmodel
            let view_model_class            = (**self.target_object).get_ivar::<*mut Class>("_viewModelClass");
            let new_view_model: *mut Object = msg_send!(*view_model_class, alloc);
            let new_view_model: *mut Object = msg_send!(new_view_model, init);
            let new_view_model              = StrongPtr::new(new_view_model);

            // Store in the list of viewmodels
            self.viewmodels.insert(viewmodel_id, new_view_model);
        }
    }

    ///
    /// Performs a viewmodel action
    ///
    fn dispatch_viewmodel_action(&self, viewmodel: &StrongPtr, action: ViewModelAction) {
        unsafe {
            use self::ViewModelAction::*;

            match action {
                CreateProperty(property_id)             => { msg_send!(**viewmodel, setNothing: property_id as u64); }
                SetPropertyValue(property_id, value)    => { msg_send!(**viewmodel, setProperty: property_id as u64 toValue: &*FloProperty::from(value)); }
            }
        }
    }

    ///
    /// Returns the FloProperty object representing the specified UI property
    ///
    fn flo_property(&self, property: AppProperty) -> FloProperty {
        use self::AppProperty::*;

        match property {
            Nothing                         => FloProperty::from(PropertyValue::Nothing),
            Bool(val)                       => FloProperty::from(PropertyValue::Bool(val)),
            Int(val)                        => FloProperty::from(PropertyValue::Int(val)),
            Float(val)                      => FloProperty::from(PropertyValue::Float(val)),
            String(val)                     => FloProperty::from(PropertyValue::String(val)),

            Bind(viewmodel_id, property_id) => {
                let viewmodel = self.viewmodels.get(&viewmodel_id);

                if let Some(viewmodel) = viewmodel {
                    unsafe { FloProperty::binding(property_id, **viewmodel) }
                } else {
                    FloProperty::from(PropertyValue::String("ViewModel not found".to_string()))
                }
            }
        }
    }
}

/// WeakPtr is not Send because Object is not Send... but we need to be able to send objective-C objects between threads so
/// we can schedule on the main thread and they are thread-safe at least in objective C itself, so let's assume this is
/// an oversight for now.
unsafe impl Send for CocoaSession { }
unsafe impl Send for NotifyRef { }

impl CocoaSessionNotify {
    ///
    /// Creates a notifier for the specified object
    ///
    pub fn new(obj: &StrongPtr) -> CocoaSessionNotify {
        CocoaSessionNotify {
            notify_object: Mutex::new(
                NotifyRef { target_object: obj.weak() }
            )
        }
    }
}

impl executor::Notify for CocoaSessionNotify {
    fn notify(&self, _: usize) {
        // Load the target object
        let target_object = self.notify_object.lock().unwrap();

        // If it still exists, send the message to the object on the main thread
        unsafe {
            autoreleasepool(move || {
                let target_object = target_object.target_object.load();

                if *target_object != nil {
                    let modes: *mut Object  = msg_send!(class!(NSMutableArray), alloc);
                    let modes               = msg_send!(modes, init);
                    let modes               = StrongPtr::new(modes);

                    msg_send!(*modes, addObject: NSDefaultRunLoopMode);
                    msg_send!(*modes, addObject: NSModalPanelRunLoopMode);
                    msg_send!(*modes, addObject: NSEventTrackingRunLoopMode);

                    msg_send![*target_object, performSelectorOnMainThread: sel!(actionStreamReady) withObject: nil waitUntilDone: NO modes: modes];
                }
            });
        }
    }
}
