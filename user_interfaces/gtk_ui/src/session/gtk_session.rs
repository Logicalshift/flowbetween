use super::viewmodel::*;
use super::attributes::*;
use super::action_sink::*;
use super::gtk_control::*;
use super::property_action::*;
use super::gtk_user_interface::*;
use super::consolidate_actions::*;
use super::super::gtk_event::*;
use super::super::gtk_action::*;
use super::super::gtk_widget_event_type::*;

use flo_ui::*;
use flo_ui::session::*;
use flo_stream::*;
use ::desync::*;

use gtk;
use futures::*;
use futures::future::{BoxFuture, LocalBoxFuture};
use std::mem;
use std::rc::*;
use std::sync::*;
use std::collections::{HashMap, HashSet};

///
/// Core data structures associated with a Gtk session
///
struct GtkSessionCore {
    /// The ID to assign to the next widget generated for this session
    next_widget_id: i64,

    /// The root Gtk control
    root_control: Option<GtkControl>,

    /// The GTK user interface
    gtk_ui: GtkUserInterface,

    /// The viewmodel for this session
    viewmodel: GtkSessionViewModel,

    /// Specifies the controller path for particular widget IDs
    controller_for_widget: HashMap<WidgetId, Rc<Vec<String>>>,

    /// Maps canvas names (controller and canvas name) to the widgets that they're being drawn upon
    widgets_for_canvas: HashMap<(Rc<Vec<String>>, String), HashSet<WidgetId>>
}

///
/// The Gtk session object represents a session running with Gtk
///
pub struct GtkSession<Ui> {
    /// Core data structures for the GTK session
    core:       Arc<Mutex<GtkSessionCore>>,

    /// The core UI that this session is running
    core_ui:    Ui
}

impl<Ui: CoreUserInterface> GtkSession<Ui> {
    ///
    /// Creates a new session connecting a core UI to a Gtk UI
    ///
    pub fn new(core_ui: Ui, gtk_ui: GtkUserInterface) -> GtkSession<Ui> {
        // Get the GTK event streams
        let mut gtk_action_sink     = Arc::new(Desync::new(gtk_ui.get_input_sink()));

        // Create the main window (always ID 0)
        Self::create_main_window(&mut gtk_action_sink);

        // Create the viewmodel (which gets its own input sink)
        let viewmodel = GtkSessionViewModel::new();

        // Create the core
        let core = GtkSessionCore {
            next_widget_id:         0,
            root_control:           None,
            gtk_ui:                 gtk_ui,
            viewmodel:              viewmodel,
            controller_for_widget:  HashMap::new(),
            widgets_for_canvas:     HashMap::new()
        };
        let core = Arc::new(Mutex::new(core));

        // Finish up by creating the new session
        let session     = GtkSession {
            core:       core,
            core_ui:    core_ui
        };

        session
    }

    ///
    /// Runs this session until it finishes
    ///
    pub async fn run(self) {
        // Create the processors
        let action_process      = self.create_action_process();
        let event_process       = self.create_event_process();

        // Run until the window is closed (or the any of the processing streams are closed)
        let close_window        = self.when_window_closed();
        let run_until_closed    = future::select(close_window, action_process);
        let run_until_closed    = future::select(run_until_closed, event_process);

        // Spawn the executor
        run_until_closed.await;
    }

    ///
    /// Creates a future that will resolve when all of the windows associated with this session are closed
    ///
    pub fn when_window_closed(&self) -> BoxFuture<'static, ()> {
        // There's only window 0 at the moment
        let event_stream = self.core.lock().unwrap().gtk_ui.get_updates();

        // Filter for window close events
        let window_close_events = event_stream.filter(|evt| future::ready(evt == &Ok(GtkEvent::CloseWindow(WindowId::Assigned(0)))));

        // We want the first window close event
        let next_window_close = window_close_events.into_future();

        // Result is the window close event with the item remove
        next_window_close.map(|_| ()).boxed()
    }

    ///
    /// Creates a future that will stop when the UI stops producing events, which connects events from the
    /// core UI to the GTK UI.
    ///
    pub fn create_action_process(&self) -> LocalBoxFuture<'static, ()> {
        // These are the streams we want to connect
        let mut gtk_action_sink     = self.core.lock().unwrap().gtk_ui.get_input_sink();
        let core_updates            = self.core_ui.get_updates();

        // Map the core updates to GTK updates
        let core                    = self.core.clone();
        let mut gtk_core_updates    = core_updates
            .map(move |updates| {
                // Lock the core while we process these updates
                let mut core    = core.lock().unwrap();
                let updates     = updates.unwrap_or_else(|()| vec![]);

                // Generate all of the actions for the current set of updates
                let actions: Vec<_> = updates.into_iter()
                    .flat_map(|update| core.process_update(update))
                    .filter(|action| !action.is_no_op())
                    .collect();

                // Send as a single block to the GTK thread
                stream::iter(vec![actions])
            })
            .flatten();

        // Connect the updates to the sink to generate our future
        let action_process = async move {
            while let Some(action_list) = gtk_core_updates.next().await {
                if action_list.len() > 0 {
                    gtk_action_sink.publish(action_list).await
                }
            }
        };

        action_process.map(|_stream_sink| ()).boxed_local()
    }

    ///
    /// Creates a future that will stop when the GTK side stops producing events, which connects events from GTK
    /// to the core UI
    ///
    pub fn create_event_process(&self) -> LocalBoxFuture<'static, ()> {
        // GTK events become input events on the core side
        let gtk_events          = self.core.lock().unwrap().gtk_ui.get_updates();
        let mut core_input      = self.core_ui.get_input_sink();

        // Connect the streams
        let core                = self.core.clone();
        let core_ui_events      = gtk_events
            .map(move |event| {
                let mut core = core.lock().unwrap();

                // Generate the core UI events for this event
                event.map(|event| core.process_event(event))
                    .unwrap_or_else(|_| vec![])
            })
            .filter(|events| future::ready(events.len() > 0));
        let mut core_ui_events  = ConsolidateActionsStream::new(core_ui_events);

        // Send the processed events to the core input
        let event_process = async move {
            while let Some(input) = core_ui_events.next().await {
                core_input.publish(input).await
            }
        };

        event_process.map(|_stream_sink| ()).boxed_local()
    }

    ///
    /// Creates the main window (ID 0) to run our session in
    ///
    fn create_main_window(action_sink: &mut GtkActionSink) {
        use self::GtkAction::*;
        use self::GtkWindowAction::*;

        // Create window 0, which will be the main window where the UI will run
        publish_actions(action_sink, vec![
            Window(WindowId::Assigned(0), vec![
                New(gtk::WindowType::Toplevel),
                SetPosition(gtk::WindowPosition::Center),
                SetDefaultSize(1920, 1080),             // TODO: make configurable (?)
                SetTitle("FlowBetween".to_string()),    // TODO: make configurable
                ShowAll
            ])
        ]);
    }
}

impl GtkSessionCore {
    ///
    /// Processes a GTK event into a UI event
    ///
    pub fn process_event(&mut self, event: GtkEvent) -> Vec<UiEvent> {
        use self::GtkEvent::*;

        match event {
            None                                        => vec![],
            CloseWindow(_window_id)                     => vec![],
            Tick                                        => vec![ UiEvent::Tick ],
            Event(widget, event_name, parameter)        => self.controller_for_widget.get(&widget)
                .map(|controller| vec![ UiEvent::Action((**controller).clone(), event_name, parameter.into()) ])
                .unwrap_or(vec![])
        }
    }

    ///
    /// Processes an update from the core UI and returns the resulting GtkActions after updating
    /// the state in the core
    ///
    pub fn process_update(&mut self, update: UiUpdate) -> Vec<GtkAction> {
        use self::UiUpdate::*;

        match update {
            Start                                   => vec![],
            UpdateUi(ui_differences)                => self.update_ui(ui_differences),
            UpdateCanvas(canvas_differences)        => self.update_canvases(canvas_differences),
            UpdateViewModel(viewmodel_differences)  => self.update_viewmodel(viewmodel_differences)
        }
    }

    ///
    /// Creates an ID for a widget in this core
    ///
    fn create_widget_id(&mut self) -> WidgetId {
        let widget_id = self.next_widget_id;
        self.next_widget_id += 1;
        WidgetId::Assigned(widget_id)
    }

    ///
    /// Given a set of actions with viewmodel dependencies, translates them into standard Gtk action while
    /// binding them into the viewmodel for this control
    ///
    fn bind_viewmodel(&mut self, control_id: WidgetId, controller_path: Rc<Vec<String>>, actions: Vec<PropertyWidgetAction>) -> Vec<GtkAction> {
        use self::PropertyAction::*;

        let viewmodel = &mut self.viewmodel;

        vec![
            GtkAction::Widget(control_id,
                actions.into_iter()
                    .flat_map(|action| {
                        match action {
                            Unbound(action)     => vec![action],
                            Bound(prop, map_fn) => viewmodel.bind(control_id, &*controller_path, &prop, map_fn)
                        }
                    })
                    .collect()
            )
        ]
    }

    ///
    /// Generates the actions to create a particular control, and binds it to the viewmodel to keep it up to
    /// date
    ///
    fn create_control(&mut self, control: &Control, controller_path: Rc<Vec<String>>, is_root: bool) -> (GtkControl, Vec<GtkAction>) {
        // Assign an ID for this control
        let control_id      = self.create_widget_id();
        let mut gtk_control = GtkControl::new(control_id, control.controller().map(|controller| controller.to_string()));

        // Associate the controller path with the new id
        self.controller_for_widget.insert(control_id, Rc::clone(&controller_path));

        // Get the actions to create this control
        let mut create_this_control = control.to_gtk_actions();

        // For the root control only we want to create a layout rather than a fixed container (so Gtk will let the user shrink the window)
        if is_root {
            create_this_control = create_this_control.into_iter()
                .map(|action| {
                    match action {
                        PropertyAction::Unbound(GtkWidgetAction::New(GtkWidgetType::Fixed))     |
                        PropertyAction::Unbound(GtkWidgetAction::New(GtkWidgetType::Overlay))   => {
                            PropertyAction::Unbound(GtkWidgetAction::New(GtkWidgetType::Layout))
                        }

                        other => other
                    }
                })
                .collect();
        }

        // Bind any properties to the view model
        let mut create_this_control = self.bind_viewmodel(control_id, Rc::clone(&controller_path), create_this_control);

        // Work out the controller path for the subcomponents
        // If a control has a controller attribute, it's not part of that controller, but its subcomponents are
        let subcomponents_controller_path = if let Some(controller) = control.controller() {
            let mut new_controller_path = (*controller_path).clone();
            new_controller_path.push(controller.to_string());

            Rc::new(new_controller_path)
        } else {
            Rc::clone(&controller_path)
        };

        // Add the actions to create any subcomponent
        let mut subcomponent_ids = vec![];
        for subcomponent in control.subcomponents().unwrap_or(&vec![]) {
            // Create the subcomponent
            let (subcomponent, create_subcomponent) = self.create_control(subcomponent, Rc::clone(&subcomponents_controller_path), false);

            // Store as a child control
            subcomponent_ids.push(subcomponent.widget_id);
            gtk_control.child_controls.push(subcomponent);
            create_this_control.extend(create_subcomponent);
        }

        // Add in the subcomponents for this control
        if subcomponent_ids.len() > 0 {
            create_this_control.push(GtkAction::Widget(control_id, vec![ GtkWidgetAction::Content(WidgetContent::SetChildren(subcomponent_ids)) ]));
        }

        // If this control has a canvas, then store it in the 'widgets for canvas' structure so we'll send updates there
        if let Some(canvas) = control.canvas_resource() {
            let canvas_name             = canvas.name().unwrap_or_else(|| canvas.id().to_string());
            let canvas_id               = (Rc::clone(&controller_path), canvas_name);
            let widgets_for_canvas      = self.widgets_for_canvas.entry(canvas_id)
                .or_insert_with(|| HashSet::new());

            widgets_for_canvas.insert(control_id);
        }

        // Wire up any events this control might have registered
        let wire_actions = self.wire_events_for_control(control);
        if wire_actions.len() > 0 {
            create_this_control.push(GtkAction::Widget(control_id, wire_actions));
        }

        // Result is the control ID and the actions required to create this control and its subcomponents
        (gtk_control, create_this_control)
    }

    ///
    /// Removes the controller path for a particular control and any child controls it might have
    ///
    fn remove_controller_path(&mut self, control: &GtkControl) {
        // Remove the child controls too
        control.child_controls.iter().for_each(|control| self.remove_controller_path(control));

        // Remove this control
        self.controller_for_widget.remove(&control.widget_id);
    }

    ///
    /// Removes a control and it's child controls from the session data structures, and generates
    /// the actions needed to remove it from the GTK control hierarchy.
    ///
    fn delete_control(&mut self, control: &GtkControl) -> Vec<GtkAction> {
        // Remove the controller path for this control
        self.remove_controller_path(control);

        // Remove from the widget_for_canvas hashmap
        let remove_widget_ids = control.tree_ids();
        let remove_widget_ids: HashSet<_> = remove_widget_ids.into_iter().collect();

        self.widgets_for_canvas.iter_mut()
            .for_each(|(_canvas, widgets)| widgets.retain(|id| !remove_widget_ids.contains(id)));
        self.widgets_for_canvas.retain(|_canvas, widgets| widgets.len() > 0);

        // Unbind this control from the viewmodel
        control.delete_from_viewmodel(&mut self.viewmodel);

        // Delete the control from the Gtk tree
        control.delete_actions()
    }

    ///
    /// Reads the controller path for a particular address
    ///
    fn controller_path_for_address(&self, address: &Vec<u32>) -> Vec<String> {
        let mut path            = vec![];
        let mut current_control = self.root_control.as_ref();

        for index in address {
            let index = *index;

            // Push the next entry in the controller path
            if let Some(controller) = current_control.and_then(|control| control.controller.as_ref()) {
                path.push(controller.clone());
            }

            // Get the next control
            current_control = current_control.and_then(|control| control.child_at_index(index));
        }

        // Controllers apply to the controls underneath the one that specifies a controller attribute so we don't push the last component
        path
    }

    ///
    /// Finds the control at the specified address (if there is one)
    ///
    fn control_at_address_mut<'a>(&'a mut self, address: &Vec<u32>) -> Option<&'a mut GtkControl> {
        // The control at vec![] is the root control
        let mut current_control = self.root_control.as_mut();

        // For each part of the index, the next control is just the child control at this index
        for index in address.iter() {
            current_control = current_control.and_then(|control| control.child_at_index_mut(*index));
        }

        // Result is the current control if we found one at this address
        current_control
    }

    ///
    /// Updates the control tree to add the specified control at the given address and returns
    /// the Gtk actions required to update the control children
    ///
    fn replace_control(&mut self, address: &Vec<u32>, new_control: GtkControl) -> Vec<GtkAction> {
        if address.len() == 0 {
            // We're updating the root control

            // Actions to remove the existing root control
            let delete_actions = self.root_control
                .take()
                .map(|control| self.delete_control(&control))
                .unwrap_or(vec![]);

            // Actions to set our new control as root
            let set_as_root = vec![
                GtkAction::Widget(new_control.widget_id, vec![ GtkWidgetAction::SetRoot(WindowId::Assigned(0)) ])
            ];

            // New control is now root
            self.root_control = Some(new_control);

            // Set the new root then delete the old control tree
            set_as_root.into_iter()
                .chain(delete_actions)
                .collect()
        } else {
            // We're updating a child of an existing control

            // Get the parent address
            let mut parent_address  = address.clone();
            let replace_index       = parent_address.pop().unwrap();

            // Attempt to fetch the parent
            let mut control_to_delete   = new_control;
            let update_control_tree;
            if let Some(parent) = self.control_at_address_mut(&parent_address) /* && parent.child_controls.len() < replace_index */ {
                // Parent exists and the child control is available for deletion

                // Swap out the control in the parent item
                mem::swap(&mut control_to_delete, &mut parent.child_controls[replace_index as usize]);

                // Action is to replace the children of the parent control
                let new_child_ids = parent.child_controls.iter()
                    .map(|child_control| child_control.widget_id)
                    .collect();

                update_control_tree = vec![
                    GtkAction::Widget(parent.widget_id, vec![ GtkWidgetAction::Content(WidgetContent::SetChildren(new_child_ids)) ])
                ];
            } else {
                // Oops, cannot replace the control here
                // We just generate the actions to delete the new control
                update_control_tree = vec![];
            }

            // Delete the old control
            let delete_old = self.delete_control(&control_to_delete);

            // Update the control tree then delete the old control
            update_control_tree.into_iter()
                .chain(delete_old)
                .collect()
        }
    }

    ///
    /// Generates the actions to update the UI with a particular diff
    ///
    fn update_ui_with_diff(&mut self, diff: UiDiff) -> Vec<GtkAction> {
        let controller_path = self.controller_path_for_address(&diff.address);

        // Create the actions to generate the control in this diff
        let (new_control, new_control_actions) = self.create_control(&diff.new_ui, Rc::new(controller_path), diff.address.len() == 0);

        // Replace the control at the specified address with our new control
        let replace_actions = self.replace_control(&diff.address, new_control);

        // Generate the new control then replace the old control
        new_control_actions.into_iter()
            .chain(replace_actions)
            .collect()
    }

    ///
    /// Updates the user interface with the specified set of differences
    ///
    fn update_ui(&mut self, ui_differences: Vec<UiDiff>) -> Vec<GtkAction> {
        ui_differences.into_iter()
            .flat_map(|diff| self.update_ui_with_diff(diff))
            .collect()
    }

    ///
    /// Updates the user interface with the specified set of viewmodel changes
    ///
    fn update_viewmodel(&mut self, viewmodel_differences: Vec<ViewModelUpdate>) -> Vec<GtkAction> {
        // Process the updates in the viewmodel, and return the resulting updates
        self.viewmodel.update(viewmodel_differences)
    }

    ///
    /// Updates a single canvas
    ///
    fn update_canvas(&mut self, canvas_difference: CanvasDiff) -> Vec<GtkAction> {
        let controller_path = Rc::new(canvas_difference.controller);
        let canvas_name     = canvas_difference.canvas_name;
        let updates         = canvas_difference.updates;
        let canvas_id       = (controller_path, canvas_name);

        let widgets         = self.widgets_for_canvas.get(&canvas_id);

        if let Some(widgets) = widgets {
            // Generate updates for all of the widgets with these canvases
            if widgets.len() == 1 {
                // One widget gets the actions (we can move them instead of copying)
                let widget_id = widgets.iter().nth(0).unwrap();
                vec![ GtkAction::Widget(*widget_id, vec![ WidgetContent::Draw(updates).into() ])]
            } else {
                // Clone the actions to many widgets
                widgets.iter().map(|widget_id| GtkAction::Widget(*widget_id, vec![ WidgetContent::Draw(updates.clone()).into() ]))
                    .collect()
            }
        } else {
            // No canvases are attached at these addresses
            vec![]
        }
    }

    ///
    /// Updates some canvases
    ///
    fn update_canvases(&mut self, canvas_differences: Vec<CanvasDiff>) -> Vec<GtkAction> {
        canvas_differences.into_iter()
            .flat_map(|diff| self.update_canvas(diff))
            .collect()
    }

    ///
    /// Generates the actions required to wire up the events for a control
    ///
    fn wire_events_for_control(&mut self, control: &Control) -> Vec<GtkWidgetAction> {
        use self::ControlAttribute::Action;
        use self::ActionTrigger::*;
        use self::GtkWidgetAction::RequestEvent;

        // Get the action attributes from the control
        let actions = control.attributes()
            .filter(|attribute| match attribute { &&Action(_, _) => true, _ => false })
            .map(|action| match action {
                &Action(ref trigger, ref name)  => (trigger.clone(), name.clone()),
                _                               => panic!("Action filter failed")
            });

        // Generate 'wire' events from them
        actions
            .flat_map(|(action, action_name)| {
                match action {
                    Click                           => vec![ RequestEvent(GtkWidgetEventType::Click, action_name) ],
                    Dismiss                         => vec![ RequestEvent(GtkWidgetEventType::Dismiss, action_name) ],
                    Paint(device)                   => vec![ RequestEvent(GtkWidgetEventType::Paint(device.into()), action_name) ],
                    Drag                            => vec![ RequestEvent(GtkWidgetEventType::Drag, action_name) ],
                    Focused                         => vec![ /* TODO */ ],
                    CancelEdit                      => vec![ /* TODO */ ],
                    EditValue                       => vec![ RequestEvent(GtkWidgetEventType::EditValue, action_name) ],
                    SetValue                        => vec![ RequestEvent(GtkWidgetEventType::SetValue, action_name) ],
                    VirtualScroll(width, height)    => vec![ RequestEvent(GtkWidgetEventType::VirtualScroll(width, height), action_name) ]
                }
            })
            .collect()
    }
}
