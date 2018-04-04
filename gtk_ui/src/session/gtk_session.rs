use super::viewmodel::*;
use super::attributes::*;
use super::gtk_control::*;
use super::property_action::*;
use super::gtk_user_interface::*;
use super::super::gtk_event::*;
use super::super::gtk_action::*;
use super::super::gtk_widget_event_type::*;

use flo_ui::*;
use flo_ui::session::*;

use gtk;
use futures::*;
use futures::executor;
use futures::stream::*;
use std::mem;
use std::sync::*;

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
    viewmodel: GtkSessionViewModel
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
        let mut gtk_action_sink     = gtk_ui.get_input_sink();

        // Create the main window (always ID 0)
        Self::create_main_window(&mut gtk_action_sink);

        // Create the viewmodel (which gets its own input sink)
        let viewmodel = GtkSessionViewModel::new();

        // Create the core
        let core = GtkSessionCore {
            next_widget_id: 0,
            root_control:   None,
            gtk_ui:         gtk_ui,
            viewmodel:      viewmodel
        };
        let core = Arc::new(Mutex::new(core));

        // Finish up by creating the new session
        GtkSession {
            core:       core,
            core_ui:    core_ui
        }
    }

    ///
    /// Runs this session until it finishes
    /// 
    pub fn run(self) {
        // Create the processors
        let action_process      = self.create_action_process();
        let event_process       = self.create_event_process();
        
        // Run until the window is closed (or the any of the processing streams are closed)
        let close_window        = self.when_window_closed();
        let run_until_closed    = close_window
            .select2(action_process)
            .select2(event_process)
            .map_err(|_| ());

        // Spawn the executor
        let mut runner = executor::spawn(run_until_closed);

        // Wait for everything to run
        runner.wait_future().unwrap();
    }

    ///
    /// Creates a future that will resolve when all of the windows associated with this session are closed
    /// 
    pub fn when_window_closed(&self) -> Box<Future<Item=(), Error=()>> {
        // There's only window 0 at the moment
        let event_stream = self.core.lock().unwrap().gtk_ui.get_updates();

        // Filter for window close events
        let window_close_events = event_stream.filter(|evt| evt == &GtkEvent::CloseWindow(WindowId::Assigned(0)));

        // We want the first window close event
        let next_window_close = window_close_events.into_future();

        // Result is the window close event with the item remove
        Box::new(next_window_close.map(|_| ()).map_err(|_| ()))
    }

    ///
    /// Creates a future that will stop when the UI stops producing events, which connects events from the
    /// core UI to the GTK UI.
    /// 
    pub fn create_action_process(&self) -> Box<Future<Item=(), Error=()>> {
        // These are the streams we want to connect
        let gtk_action_sink     = self.core.lock().unwrap().gtk_ui.get_input_sink();
        let core_updates        = self.core_ui.get_updates();

        // Map the core updates to GTK updates
        let core                = self.core.clone();
        let gtk_core_updates    = core_updates
            .map(move |updates| {
                // Lock the core while we process these updates
                let mut core = core.lock().unwrap();

                // Generate all of the actions for the current set of updates
                let actions: Vec<_> = updates.into_iter()
                    .flat_map(|update| core.process_update(update))
                    .filter(|action| !action.is_no_op())
                    .collect();
                
                // Send as a single block to the GTK thread
                iter_ok(vec![actions])
            })
            .flatten();
        
        // Connect the updates to the sink to generate our future
        let action_process = gtk_action_sink.send_all(gtk_core_updates.filter(|action_list| action_list.len() > 0));

        Box::new(action_process.map(|_stream_sink| ()))
    }

    ///
    /// Creates a future that will stop when the GTK side stops producing events, which connects events from GTK
    /// to the core UI
    ///
    pub fn create_event_process(&self) -> Box<Future<Item=(), Error=()>> {
        // GTK events become input events on the core side
        let gtk_events  = self.core.lock().unwrap().gtk_ui.get_updates();
        let core_input  = self.core_ui.get_input_sink();

        // Connect the streams
        let core            = self.core.clone();
        let core_ui_events  = gtk_events
            .map(move |event| {
                let mut core = core.lock().unwrap();

                // Generate the core UI events for this event
                core.process_event(event)
            });
        
        // Send the processed events to the core input
        let event_process = core_input.send_all(core_ui_events);

        Box::new(event_process.map(|_stream_sink| ()))
    }

    ///
    /// Creates the main window (ID 0) to run our session in
    /// 
    fn create_main_window<S: Sink<SinkItem=Vec<GtkAction>, SinkError=()>>(action_sink: &mut S) {
        use self::GtkAction::*;
        use self::GtkWindowAction::*;    

        // Create window 0, which will be the main window where the UI will run
        action_sink.start_send(vec![
            Window(WindowId::Assigned(0), vec![
                New(gtk::WindowType::Toplevel),
                SetPosition(gtk::WindowPosition::Center),
                SetDefaultSize(1920, 1080),             // TODO: make configurable (?)
                SetTitle("FlowBetween".to_string()),    // TODO: make configurable
                ShowAll
            ])
        ]).unwrap();
    }
}

impl<CoreController: Controller+'static> GtkSession<UiSession<CoreController>> {
    ///
    /// Creates a GTK session from a core controller
    /// 
    pub fn from(controller: CoreController, gtk_ui: GtkUserInterface) -> GtkSession<UiSession<CoreController>> {
        let session = UiSession::new(controller);
        Self::new(session, gtk_ui)
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
            CloseWindow(WindowId)                       => vec![],
            Tick                                        => vec![ UiEvent::Tick ],
            Event(WidgetId, String, GtkEventParameter)  => vec![]
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
            UpdateCanvas(canvas_differences)        => vec![],
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
    fn bind_viewmodel(&mut self, control_id: WidgetId, controller_path: &Vec<String>, actions: Vec<PropertyWidgetAction>) -> Vec<GtkAction> {
        use self::PropertyAction::*;

        let viewmodel = &mut self.viewmodel;
        
        vec![
            GtkAction::Widget(control_id, 
                actions.into_iter()
                    .flat_map(|action| {
                        match action {
                            Unbound(action)     => vec![action],
                            Bound(prop, map_fn) => viewmodel.bind(control_id, controller_path, &prop, map_fn)
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
    fn create_control(&mut self, control: &Control, controller_path: &Vec<String>) -> (GtkControl, Vec<GtkAction>) {
        // Assign an ID for this control
        let control_id      = self.create_widget_id();
        let mut gtk_control = GtkControl::new(control_id, control.controller().map(|controller| controller.to_string()));

        // Get the actions to create this control
        let create_this_control = control.to_gtk_actions();

        // Bind any properties to the view model
        let mut create_this_control = self.bind_viewmodel(control_id, controller_path, create_this_control);

        // Work out the controller path for the subcomponents
        // If a control has a controller attribute, it's not part of that controller, but its subcomponents are
        let mut new_controller_path;

        let subcomponents_controller_path = if let Some(controller) = control.controller() {
            new_controller_path = controller_path.clone();
            new_controller_path.push(controller.to_string());

            &new_controller_path
        } else {
            controller_path
        };

        // Add the actions to create any subcomponent
        let mut subcomponent_ids = vec![];
        for subcomponent in control.subcomponents().unwrap_or(&vec![]) {
            // Create the subcomponent
            let (subcomponent, create_subcomponent) = self.create_control(subcomponent, subcomponents_controller_path);

            // Store as a child control
            subcomponent_ids.push(subcomponent.widget_id);
            gtk_control.child_controls.push(subcomponent);
            create_this_control.extend(create_subcomponent);
        }

        // Add in the subcomponents for this control
        if subcomponent_ids.len() > 0 {
            create_this_control.push(GtkAction::Widget(control_id, vec![ GtkWidgetAction::Content(WidgetContent::SetChildren(subcomponent_ids)) ]));
        }

        // Add in any events this control might have registered
        let wire_actions = self.wire_events_for_control(control);
        if wire_actions.len() > 0 {
            create_this_control.push(GtkAction::Widget(control_id, wire_actions));
        }

        // Result is the control ID and the actions required to create this control and its subcomponents
        (gtk_control, create_this_control)
    }

    ///
    /// Generates the actions required to delete a particular control
    /// 
    fn delete_control(&mut self, control: &GtkControl) -> Vec<GtkAction> {
        // Unbind this control from the viewmodel
        control.delete_from_viewmodel(&mut self.viewmodel);

        // Delete the control from the Gtk tree
        control.delete_actions()
    }

    ///
    /// Finds the control at the specified address (if there is one)
    /// 
    fn control_at_address<'a>(&'a self, address: &Vec<u32>) -> Option<&'a GtkControl> {
        // The control at vec![] is the root control
        let mut current_control = self.root_control.as_ref();

        // For each part of the index, the next control is just the child control at this index
        for index in address.iter() {
            current_control = current_control.and_then(|control| control.child_at_index(*index));
        }

        // Result is the current control if we found one at this address
        current_control
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
        let (new_control, new_control_actions) = self.create_control(&diff.new_ui, &controller_path);

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
                    Dismiss                         => vec![],
                    Paint(device)                   => vec![],
                    Drag                            => vec![],
                    EditValue                       => vec![],
                    SetValue                        => vec![],
                    VirtualScroll(width, height)    => vec![]
                }
            })
            .collect()
    }
}
