use flo_binding::*;
use flo_binding::binding_context::*;

use super::image::*;
use super::control::*;
use super::viewmodel::*;
use super::binding_canvas::*;
use super::resource_manager::*;

use futures::future::{BoxFuture};

use std::sync::*;
use std::collections::{HashSet, HashMap};

// TODO: it would be nice to use specific enum types for
// sub-controllers and events. However, this causes a few
// problems. Firstly, it's pretty tricky to add them as
// parameters for Control due to the need to be clonable,
// serializable, etc. Secondly, it would probably mean that
// all controllers and sub-controllers would need the same
// type (I suppose there could be some kind of controller
// that takes two controllers and makes them the same
// type but it's still a lot of work just for a slightly
// nicer system)
//
// Can't use Any as it's not really compatible with cloning
// and serialization
//
// For this reason, events and subcontrollers are identified
// by strings which need to be parsed.

///
/// Controllers represent a portion of the UI and provide a hub for
/// receiving events related to it and connecting the model or
/// viewmodel.
///
pub trait Controller : Send+Sync {
    ///
    /// Retrieves a Control representing the UI for this controller
    ///
    fn ui(&self) -> BindRef<Control>;

    ///
    /// Retrieves the viewmodel for this controller
    ///
    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> { None }

    ///
    /// Attempts to retrieve a sub-controller of this controller
    ///
    fn get_subcontroller(&self, _id: &str) -> Option<Arc<dyn Controller>> { None }

    ///
    /// Callback for when a control associated with this controller generates an action
    ///
    fn action(&self, _action_id: &str, _action_data: &ActionParameter) { }

    ///
    /// Retrieves a resource manager containing the images used in the UI for this controller
    ///
    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> { None }

    ///
    /// Retrieves a resource manager containing the canvases used in the UI for this controller
    ///
    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> { None }

    ///
    /// Returns a future representing the run-time for this controller
    ///
    /// This is run in sync with the main UI thread: ie, all controllers that have a future must
    /// be asleep before a tick can pass. This also provides a way for a controller to wake the
    /// run-time thread.
    ///
    fn runtime(&self) -> Option<BoxFuture<'static, ()>> { None }

    ///
    /// Called just before an update is processed
    ///
    /// This is called for every controller every time after processing any actions
    /// that might have occurred.
    ///
    fn tick(&self) { }
}

/// Controller that provides just an empty control
pub struct EmptyController;

impl Controller for EmptyController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::from(bind(Control::empty()))
    }
}

///
/// Returns the full UI tree for the current state of a controller
///
fn get_full_ui_tree(base_controller: &Arc<dyn Controller>) -> Control {
    let base_ui = base_controller.ui();

    base_ui.get().map(&|control| {
        if let Some(controller_id) = control.controller() {
            // Has a subcontroller. These are retrieved without a binding context to allow them to initialise themselves and any dependent bindings if needed
            let subcontroller = BindingContext::out_of_context(|| { base_controller.get_subcontroller(controller_id) });

            if let Some(subcontroller) = subcontroller {
                // If we can look up the subcontroller then this control should have its UI as its subcomponents
                let subassembly = get_full_ui_tree(&subcontroller);
                control.clone().with(vec![subassembly])
            } else {
                // No subcontroller
                control.clone()
            }
        } else {
            // Control is untouched
            control.clone()
        }
    })
}

///
/// Given an address (a list of subcomponent node indices), finds the
/// path through the controllers that will supply the controller that
/// owns that node.
///
pub fn controller_path_for_address<'a>(ui_tree: &'a Control, address: &Vec<u32>) -> Option<Vec<&'a str>> {
    let mut result          = vec![];
    let mut current_node    = ui_tree;

    for index in address.iter() {
        // Append the controller to the path
        if let Some(controller) = current_node.controller() {
            result.push(controller);
        }

        // Move to the next node along this address
        if let Some(subcomponents) = current_node.subcomponents() {
            if (*index as usize) < subcomponents.len() {
                current_node = &subcomponents[*index as usize];
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    Some(result)
}

///
/// Returns a bound control that expands the content of any
/// sub-controllers that might be present.
///
/// Note that this only maintains a weak reference to the
/// controller, so if this is the last reference, it will
/// return an empty UI instead.
///
pub fn assemble_ui(base_controller: Arc<dyn Controller>) -> BindRef<Control> {
    // We keep a weak reference to the controller so the binding will not hold on to it
    let weak_controller = Arc::downgrade(&base_controller);

    // Result is computed
    return BindRef::from(computed(move || {
        if let Some(base_controller) = weak_controller.upgrade() {
            get_full_ui_tree(&base_controller)
        } else {
            // Controller is no longer around, so it doesn't have a UI any more
            Control::empty()
        }
    }));
}

///
/// Retrieves the list of supported commands that can trigger actions for a controller, and the
/// list of subcontrollers that might have further commands to process
///
fn get_supported_commands(controller: &Arc<dyn Controller>, path: &Vec<String>) -> (Vec<CommandBinding>, HashSet<String>) {
    // Retrieve the UI for the control
    let ui                  = controller.ui().get();

    // Process the controls to find the command attributes
    let mut commands        = vec![];
    let mut subcontrollers  = HashSet::new();
    let mut remaining       = vec![&ui];

    while let Some(control) = remaining.pop() {
        for attr in control.attributes() {
            match attr {
                ControlAttribute::Controller(controller_name)                   => { subcontrollers.insert(controller_name.clone()); }
                ControlAttribute::SubComponents(subcomponents)                  => { remaining.extend(subcomponents.iter()); }
                ControlAttribute::Action(ActionTrigger::Command(cmd), action)   => { commands.push(CommandBinding { command: cmd.clone(), controller: Arc::downgrade(controller), path: path.clone(), action: action.clone() }) }

                _                                                               => { }
            }
        }
    }

    (commands, subcontrollers)
}

///
/// Creates a binding of a map of commands to the controller paths that respond to those commands
///
pub fn command_map_binding(controller: Arc<dyn Controller>) -> BindRef<Arc<HashMap<Command, Vec<CommandBinding>>>> {
    let controller  = Arc::downgrade(&controller);
    let binding     = computed(move || {
        // Fetch the controller if it hasn't been released
        let controller = controller.upgrade();
        let controller = if let Some(controller) = controller { controller } else { return Arc::new(HashMap::new()); };

        // Generate a hashmap with a list of all the controller paths
        let mut controllers_for_command = HashMap::new();
        let mut controllers             = vec![(controller, vec![])];

        while let Some((controller, path)) = controllers.pop() {
            // Fetch the commmands and subcontrollers for this controller
            let (commands, subcontrollers) = get_supported_commands(&controller, &path);

            // Add the commands for this path
            for cmd in commands {
                controllers_for_command.entry(cmd.command.clone())
                    .or_insert_with(|| vec![])
                    .push(cmd);
            }

            // Process the subcontrollers
            for subcontroller_name in subcontrollers {
                if let Some(subcontroller) = controller.get_subcontroller(&subcontroller_name) {
                    // Extend the path
                    let mut subcontroller_path = path.clone();
                    subcontroller_path.push(subcontroller_name);

                    // Process this controller next
                    controllers.push((subcontroller, subcontroller_path));
                }
            }
        }

        Arc::new(controllers_for_command)
    });

    BindRef::from(binding)
}

///
/// Retrieves all keybindings defined in a particular controller, along with the list of subcontrollers
///
fn get_keybindings(controller: &Arc<dyn Controller>) -> (HashMap<KeyBinding, HashSet<Command>>, HashSet<String>) {
    // Retrieve the UI for the control
    let ui                  = controller.ui().get();

    // Process the controls to find the command attributes
    let mut keybindings     = HashMap::new();
    let mut subcontrollers  = HashSet::new();
    let mut remaining       = vec![&ui];

    while let Some(control) = remaining.pop() {
        for attr in control.attributes() {
            match attr {
                ControlAttribute::Controller(controller_name)                   => { subcontrollers.insert(controller_name.clone()); }
                ControlAttribute::SubComponents(subcomponents)                  => { remaining.extend(subcomponents.iter()); }
                ControlAttribute::BindKey(key, cmd)                             => { keybindings.entry(key.clone()).or_insert_with(|| HashSet::new()).insert(cmd.clone()); }

                _                                                               => { }
            }
        }
    }

    (keybindings, subcontrollers)
}

///
/// Creates a binding mapping key presses to the commands they should generate
///
pub fn keymap_binding(controller: Arc<dyn Controller>) -> BindRef<Arc<HashMap<KeyBinding, HashSet<Command>>>> {
    let controller  = Arc::downgrade(&controller);
    let binding     = computed(move || {
        // Fetch the controller if it hasn't been released
        let controller = controller.upgrade();
        let controller = if let Some(controller) = controller { controller } else { return Arc::new(HashMap::new()); };

        // Generate a hashmap with a list of all the controller paths
        let mut all_keybindings         = HashMap::new();
        let mut controllers             = vec![controller];

        while let Some(controller) = controllers.pop() {
            // Fetch the key bindings and subcontrollers for this controller
            let (keybindings, subcontrollers) = get_keybindings(&controller);

            // Merge the keybindings into the 'all' list
            keybindings.into_iter()
                .for_each(|(key, commands)| {
                    all_keybindings.entry(key)
                        .or_insert_with(|| HashSet::new())
                        .extend(commands);
                });

            // Process the subcontrollers
            for subcontroller_name in subcontrollers {
                if let Some(subcontroller) = controller.get_subcontroller(&subcontroller_name) {
                    // Process this controller next
                    controllers.push(subcontroller);
                }
            }
        }

        Arc::new(all_keybindings)
    });

    BindRef::from(binding)
}

///
/// A controller that does nothing
///
pub struct NullController {
}

impl NullController {
    pub fn new() -> NullController {
        NullController { }
    }
}

impl Controller for NullController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::from(bind(Control::empty()))
    }

    fn get_subcontroller(&self, _id: &str) -> Option<Arc<dyn Controller>> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestController {
        pub label_controller: Arc<LabelController>,
        view_model: Arc<NullViewModel>,
        ui: BindRef<Control>
    }
    struct LabelController {
        pub label_text: Binding<String>,
        view_model: Arc<NullViewModel>,
        ui: BindRef<Control>
    }

    impl TestController {
        pub fn new() -> TestController {
            TestController {
                label_controller: Arc::new(LabelController::new()),
                view_model: Arc::new(NullViewModel::new()),
                ui: BindRef::from(bind(Control::container().with_controller("Test")))
            }
        }
    }

    impl LabelController {
        pub fn new() -> LabelController {
            let text = bind("Test".to_string());
            let label_text = text.clone();

            LabelController {
                label_text: label_text,
                view_model: Arc::new(NullViewModel::new()),
                ui: BindRef::from(computed(move || {
                    let text = text.get();

                    Control::label().with(text)
                }))
            }
        }
    }

    impl Controller for TestController {
        fn ui(&self) -> BindRef<Control> {
            BindRef::clone(&self.ui)
        }

        fn get_subcontroller(&self, _id: &str) -> Option<Arc<dyn Controller>> {
            Some(self.label_controller.clone())
        }

        fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
            Some(self.view_model.clone())
        }
    }

    impl Controller for LabelController {
        fn ui(&self) -> BindRef<Control> {
            BindRef::clone(&self.ui)
        }

        fn get_subcontroller(&self, _id: &str) -> Option<Arc<dyn Controller>> {
            None
        }

        fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
            Some(self.view_model.clone())
        }
    }

    #[test]
    fn can_assemble_simple_label() {
        let label_controller    = Arc::new(LabelController::new());
        let assembly            = assemble_ui(label_controller.clone());

        assert!(assembly.get() == Control::label().with("Test"));
    }

    #[test]
    fn can_assemble_with_subassembly() {
        let test_controller = Arc::new(TestController::new());
        let assembly        = assemble_ui(test_controller.clone());

        assert!(assembly.get() == Control::container()
            .with_controller("Test")
            .with(vec![
                Control::label().with("Test")
            ]));
    }

    #[test]
    fn label_binding_updates() {
        let label_controller    = Arc::new(LabelController::new());
        let assembly            = assemble_ui(label_controller.clone());

        assert!(assembly.get() == Control::label().with("Test"));

        label_controller.label_text.set("Changed".to_string());
        assert!(assembly.get() == Control::label().with("Changed"));
    }

    #[test]
    fn subassembly_binding_updates() {
        let test_controller = Arc::new(TestController::new());
        let assembly        = assemble_ui(test_controller.clone());

        assert!(assembly.get() == Control::container()
            .with_controller("Test")
            .with(vec![
                Control::label().with("Test")
            ]));

        test_controller.label_controller.label_text.set("Changed".to_string());

        assert!(assembly.get() == Control::container()
            .with_controller("Test")
            .with(vec![
                Control::label().with("Changed")
            ]));
    }

    #[test]
    fn subassembly_binding_updates_after_reassembly() {
        let test_controller = Arc::new(TestController::new());
        let assembly        = assemble_ui(test_controller.clone());

        assert!(assembly.get() == Control::container()
            .with_controller("Test")
            .with(vec![
                Control::label().with("Test")
            ]));

        test_controller.label_controller.label_text.set("Changed".to_string());

        let assembly        = assemble_ui(test_controller.clone());
        assert!(assembly.get() == Control::container()
            .with_controller("Test")
            .with(vec![
                Control::label().with("Changed")
            ]));
    }

    #[test]
    fn controller_path_for_empty_address_is_empty() {
        let control = Control::container()
            .with_controller("Test1")
            .with(vec![
                Control::empty(),
                Control::container()
                    .with_controller("Test2")
                    .with(vec![
                        Control::empty(),
                        Control::empty(),
                        Control::container()
                            .with_controller("Test3")
                            .with(vec![
                                Control::empty()
                            ])
                    ])
            ]);

        assert!(controller_path_for_address(&control, &vec![]).unwrap().len() == 0);
    }

    #[test]
    fn controller_path_for_specific_control_is_right() {
        let control = Control::container()
            .with_controller("Test1")
            .with(vec![
                Control::empty(),
                Control::container()
                    .with_controller("Test2")
                    .with(vec![
                        Control::empty(),
                        Control::empty(),
                        Control::container()
                            .with_controller("Test3")
                            .with(vec![
                                Control::empty()
                            ])
                    ])
            ]);

        assert!(controller_path_for_address(&control, &vec![0]) == Some(vec!["Test1"]));
    }

    #[test]
    fn controller_path_for_control_with_controller_skips_controller() {
        let control = Control::container()
            .with_controller("Test1")
            .with(vec![
                Control::empty(),
                Control::container()
                    .with_controller("Test2")
                    .with(vec![
                        Control::empty(),
                        Control::empty(),
                        Control::container()
                            .with_controller("Test3")
                            .with(vec![
                                Control::empty()
                            ])
                    ])
            ]);

        assert!(controller_path_for_address(&control, &vec![1, 2]) == Some(vec!["Test1", "Test2"]));
        assert!(controller_path_for_address(&control, &vec![1, 2, 0]) == Some(vec!["Test1", "Test2", "Test3"]));
    }
}
