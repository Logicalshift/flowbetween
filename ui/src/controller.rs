use flo_binding::*;
use flo_binding::binding_context::*;

use super::image::*;
use super::control::*;
use super::viewmodel::*;
use super::binding_canvas::*;
use super::resource_manager::*;

use std::sync::*;

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
    /// Retrieves a Control representing the UI for this controller
    fn ui(&self) -> BindRef<Control>;

    /// Retrieves the viewmodel for this controller
    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> { None }

    /// Attempts to retrieve a sub-controller of this controller
    fn get_subcontroller(&self, _id: &str) -> Option<Arc<dyn Controller>> { None }

    /// Callback for when a control associated with this controller generates an action
    fn action(&self, _action_id: &str, _action_data: &ActionParameter) { }

    /// Retrieves a resource manager containing the images used in the UI for this controller
    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> { None }

    /// Retrieves a resource manager containing the canvases used in the UI for this controller
    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> { None }

    /// Called just before an update is processed
    ///
    /// This is called for every controller every time after processing any actions
    /// that might have occurred.
    fn tick(&self) { }
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
