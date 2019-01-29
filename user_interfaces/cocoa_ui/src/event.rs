use super::app::*;
use super::session::*;
use super::core_graphics_ffi::*;

use flo_stream::*;
use flo_cocoa_pipe::*;

use cocoa::base::nil;
use objc::*;
use objc::rc::*;
use objc::declare::*;
use objc::runtime::*;

use futures::executor;
use futures::executor::Spawn;

use num_traits::cast::*;

use std::mem;
use std::sync::*;
use std::ffi::CStr;
use std::collections::HashMap;

/// The length of time to wait between receiving an event and sending it
/// (Ideally we want to send all of the events at the start of an animation frame: however, Cocoa does not
/// provide a particularly convenient way to do this, or even a good way to get the current frame rate so we 
/// just use a hard-coded value here instead. A disadvantage of this approach will be choppy updates)
const EVENT_DELAY_SECS: f64 = 1.0 / 60.0;

lazy_static! {
    /// The events class
    pub static ref FLO_EVENTS_CLASS: &'static Class = declare_flo_events_class();

    /// Used to look up the FloEvents object associated with an instance of the FloEvents class
    static ref FLO_EVENTS_STORE: Mutex<HashMap<usize, Arc<Mutex<FloEvents>>>> = Mutex::new(HashMap::new());

    /// The next ID to assign to a FloEvents object
    static ref NEXT_FLO_EVENTS_ID: Mutex<usize> = Mutex::new(0);
}

///
/// Event target for sending events from the objective-C side to the rust side
///
pub struct FloEvents {
    /// The ID of the session that these events are for
    session_id: usize,

    /// The ID of the view that will be sending these events
    view_id: usize,

    /// Where events are published
    events_publisher: Spawn<Publisher<Vec<AppEvent>>>,

    /// Set to true if we are going to receive a callback to send the events
    queued_update: bool,

    /// The events that are waiting to be sent
    pending_events: Vec<AppEvent>
}

impl FloEvents {
    ///
    /// Creates a new FloEvents
    ///
    pub fn init(publisher: Publisher<Vec<AppEvent>>, session_id: usize, view_id: usize) -> FloEvents {
        FloEvents {
            view_id:            view_id,
            session_id:         session_id,
            events_publisher:   executor::spawn(publisher),
            queued_update:      false,
            pending_events:     vec![]
        }
    }

    ///
    /// Creates the objective-C object from a FloEvents reference
    ///
    fn object_from_events(events: Arc<Mutex<FloEvents>>) -> StrongPtr {
        unsafe {
            // Assign an ID to this events object
            let events_id = {
                let mut next_flo_events_id  = NEXT_FLO_EVENTS_ID.lock().unwrap();
                let events_id               = *next_flo_events_id;
                *next_flo_events_id         += 1;

                events_id
            };

            // Store the events away
            FLO_EVENTS_STORE.lock().unwrap().insert(events_id, events);

            // Allocate the object
            let flo_events_object: *mut Object = msg_send!(*FLO_EVENTS_CLASS, alloc);
            let flo_events_object: *mut Object = msg_send!(flo_events_object, init);

            // Set it up
            (*flo_events_object).set_ivar("_eventsId", events_id);

            StrongPtr::new(flo_events_object)
        }
    }

    ///
    /// Creates a new FloEvents object for objective-C
    ///
    pub fn create_object(publisher: Publisher<Vec<AppEvent>>, session_id: usize, view_id: usize) -> StrongPtr {
        let events = FloEvents::init(publisher, session_id, view_id);
        Self::object_from_events(Arc::new(Mutex::new(events)))
    }
}

///
/// Declares the FloEvents objective C class
///
pub fn declare_flo_events_class() -> &'static Class {
    // Create the class declaration
    let mut flo_events = ClassDecl::new("FloEvents", class!(NSObject)).unwrap();

    unsafe {
        // Add the event ID ivar
        flo_events.add_ivar::<usize>("_eventsId");

        ///
        /// Converts a NSString name into a rust string
        ///
        unsafe fn name_for_name(name: &mut Object) -> String {
            let utf8 = msg_send!(name, UTF8String);
            let utf8 = CStr::from_ptr(utf8);
            utf8.to_str()
                .map(|utf8| utf8.to_string())
                .unwrap_or_else(|err| format!("<to_str: {}>", err))
        }

        // Sends an event to the events object
        unsafe fn send_event(this: &mut Object, event: AppEvent) {
            // Fetch the rust events structure
            let events_id   = (*this).get_ivar::<usize>("_eventsId");
            let flo_events  = FLO_EVENTS_STORE.lock().unwrap().get(events_id).cloned();

            flo_events.map(|flo_events| {
                let mut flo_events = flo_events.lock().unwrap();

                // Add to the pending events list
                flo_events.pending_events.push(event);

                // Add a runloop callback to actually send the events
                if !flo_events.queued_update {
                    // Build an array of all the modes
                    let modes: *mut Object  = msg_send!(class!(NSMutableArray), alloc);
                    let modes               = msg_send!(modes, init);
                    let modes               = StrongPtr::new(modes);

                    msg_send!(*modes, addObject: NSDefaultRunLoopMode);
                    msg_send!(*modes, addObject: NSModalPanelRunLoopMode);
                    msg_send!(*modes, addObject: NSEventTrackingRunLoopMode);

                    // Call this object back after a delay
                    msg_send!(this, performSelector: sel!(finishSendingEvents) withObject: nil afterDelay: EVENT_DELAY_SECS inModes: modes);

                    flo_events.queued_update = true;
                }
            });
        }

        // Retrieves the view ID for an object
        unsafe fn get_view_id(this: &mut Object) -> Option<usize> {
            let events_id   = (*this).get_ivar::<usize>("_eventsId");
            let flo_events  = FLO_EVENTS_STORE.lock().unwrap().get(events_id).cloned();
            flo_events.map(|flo_events| flo_events.lock().unwrap().view_id)
        }

        // Retrieves the view ID for an object
        unsafe fn get_session_id(this: &mut Object) -> Option<usize> {
            let events_id   = (*this).get_ivar::<usize>("_eventsId");
            let flo_events  = FLO_EVENTS_STORE.lock().unwrap().get(events_id).cloned();
            flo_events.map(|flo_events| flo_events.lock().unwrap().session_id)
        }

        // Sends the 'click' event
        extern fn send_click(this: &mut Object, _sel: Sel, name: *mut Object) {
            unsafe {
                let view_id = get_view_id(this);
                let name    = name_for_name(&mut *name);

                if let Some(view_id) = view_id {
                    send_event(this, AppEvent::Click(view_id, name));
                }
            }
        }

        // Sends the 'virtual scroll' event
        extern fn send_virtual_scroll(this: &mut Object, _sel: Sel, name: *mut Object, left: u32, top: u32, width: u32, height: u32) {
            unsafe {
                let view_id = get_view_id(this);
                let name    = name_for_name(&mut *name);

                if let Some(view_id) = view_id {
                    send_event(this, AppEvent::VirtualScroll(view_id, name, (left, top), (width, height)));
                }
            }
        }

        // Sends the paint start event
        extern fn send_paint_start(this: &mut Object, _sel: Sel, device_id: u32, name: *mut Object, painting: AppPainting) {
            unsafe {
                let view_id = get_view_id(this);
                let name    = name_for_name(&mut *name);
                let device  = AppPaintDevice::from_u32(device_id);

                if let (Some(view_id), Some(device)) = (view_id, device) {
                    send_event(this, AppEvent::PaintStart(view_id, name, device, painting));
                }
            }
        }

        // Sends the paint continue event
        extern fn send_paint_continue(this: &mut Object, _sel: Sel, device_id: u32, name: *mut Object, painting: AppPainting) {
            unsafe {
                let view_id = get_view_id(this);
                let name    = name_for_name(&mut *name);
                let device  = AppPaintDevice::from_u32(device_id);

                if let (Some(view_id), Some(device)) = (view_id, device) {
                    send_event(this, AppEvent::PaintContinue(view_id, name, device, painting));
                }
            }
        }

        // Sends the paint finish event
        extern fn send_paint_finish(this: &mut Object, _sel: Sel, device_id: u32, name: *mut Object, painting: AppPainting) {
            unsafe {
                let view_id = get_view_id(this);
                let name    = name_for_name(&mut *name);
                let device  = AppPaintDevice::from_u32(device_id);

                if let (Some(view_id), Some(device)) = (view_id, device) {
                    send_event(this, AppEvent::PaintFinish(view_id, name, device, painting));
                }
            }
        }

        // Sends the paint cancel event
        extern fn send_paint_cancel(this: &mut Object, _sel: Sel, device_id: u32, name: *mut Object, painting: AppPainting) {
            unsafe {
                let view_id = get_view_id(this);
                let name    = name_for_name(&mut *name);
                let device  = AppPaintDevice::from_u32(device_id);

                if let (Some(view_id), Some(device)) = (view_id, device) {
                    send_event(this, AppEvent::PaintCancel(view_id, name, device, painting));
                }
            }
        }

        // Redraws the canvas for the view
        extern fn redraw_canvas(this: &mut Object, _sel: Sel, size: CGSize, bounds: CGRect) {
            unsafe {
                let session_id  = get_session_id(this);
                let view_id     = get_view_id(this);

                if let (Some(session_id), Some(view_id)) = (session_id, view_id) {
                    if let Some(session) = get_cocoa_session_with_id(session_id) {
                        session.lock().unwrap().redraw_canvas_for_view(view_id, size, bounds);
                    }
                }
            }
        }

        // Clears the list of pending events
        extern fn finish_sending_events(this: &mut Object, _sel: Sel) {
            unsafe {
                // Fetch the rust events structure
                let events_id   = (*this).get_ivar::<usize>("_eventsId");
                let flo_events  = FLO_EVENTS_STORE.lock().unwrap().get(events_id).cloned();

                flo_events.map(|flo_events| {
                    let mut flo_events  = flo_events.lock().unwrap();
                    let mut pending     = vec![];

                    // Fetch and clear the list of pending events
                    mem::swap(&mut pending, &mut flo_events.pending_events);
                    flo_events.queued_update = false;

                    // Send to the publisher
                    flo_events.events_publisher.wait_send(pending).ok();
                });
            }
        }

        // Deallocates the flo_events object
        extern fn dealloc_flo_events(this: &mut Object, _sel: Sel) {
            unsafe {
                // Remove the Rust type that we're managing
                let events_id = (*this).get_ivar::<usize>("_eventsId");
                FLO_EVENTS_STORE.lock().unwrap().remove(&events_id);
            }
        }

        // Register the events methods
        flo_events.add_method(sel!(dealloc), dealloc_flo_events as extern fn(&mut Object, Sel));
        flo_events.add_method(sel!(finishSendingEvents), finish_sending_events as extern fn(&mut Object, Sel));

        flo_events.add_method(sel!(sendClick:), send_click as extern fn(&mut Object, Sel, *mut Object));
        flo_events.add_method(sel!(sendVirtualScroll:left:top:width:height:), send_virtual_scroll as extern fn(&mut Object, Sel, *mut Object, u32, u32, u32, u32));
        flo_events.add_method(sel!(sendPaintStartForDevice:name:action:), send_paint_start as extern fn(&mut Object, Sel, u32, *mut Object, AppPainting));
        flo_events.add_method(sel!(sendPaintContinueForDevice:name:action:), send_paint_continue as extern fn(&mut Object, Sel, u32, *mut Object, AppPainting));
        flo_events.add_method(sel!(sendPaintFinishForDevice:name:action:), send_paint_finish as extern fn(&mut Object, Sel, u32, *mut Object, AppPainting));
        flo_events.add_method(sel!(sendPaintCancelForDevice:name:action:), send_paint_cancel as extern fn(&mut Object, Sel, u32, *mut Object, AppPainting));
        flo_events.add_method(sel!(redrawCanvasWithSize:viewport:), redraw_canvas as extern fn(&mut Object, Sel, CGSize, CGRect));
    }

    // Finalize the class
    flo_events.register()
}
