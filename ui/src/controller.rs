use binding::*;

use super::image::*;
use super::canvas::*;
use super::control::*;
use super::viewmodel::*;
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
    fn ui(&self) -> Arc<Bound<Control>>;

    /// Retrieves the viewmodel for this controller
    fn get_viewmodel(&self) -> Arc<ViewModel>;

    /// Attempts to retrieve a sub-controller of this controller
    fn get_subcontroller(&self, _id: &str) -> Option<Arc<Controller>> { None }

    /// Callback for when a control associated with this controller generates an action
    fn action(&self, _action_id: &str) { }

    /// Retrieves a resource manager containing the images used in the UI for this controller
    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> { None }

    /// Retrieves a resource manager containing the canvases used in the UI for this controller
    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<Canvas>>> { None }
}

///
/// Returns a bound control that expands the content of any 
/// sub-controllers that might be present.
///
/// Note that this only maintains a weak reference to the
/// controller, so if this is the last reference, it will
/// return an empty UI instead.
///
pub fn assemble_ui(base_controller: Arc<Controller>) -> Box<Bound<Control>> {
    // Fetch the UI for the controller
    let base_ui         = base_controller.ui();
    let weak_controller = Arc::downgrade(&base_controller);

    // Result is computed
    return Box::new(computed(move || {
        if let Some(base_controller) = weak_controller.upgrade() {
            base_ui.get().map(&|control| {
                if let Some(controller_id) = control.controller() {
                    // Has a subcontroller
                    let subcontroller = base_controller.get_subcontroller(controller_id);

                    if let Some(subcontroller) = subcontroller {
                        // If we can look up the subcontroller then this control should have its UI as its subcomponents
                        let subassembly = assemble_ui(subcontroller);
                        control.clone().with(vec![subassembly.get()])
                    } else {
                        // No subcontroller
                        control.clone()
                    }
                } else {
                    // Control is untouched
                    control.clone()
                }
            })
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
    view_model: Arc<NullViewModel>
}

impl NullController {
    pub fn new() -> NullController {
        NullController { view_model: Arc::new(NullViewModel::new()) }
    }
}

impl Controller for NullController {
    fn ui(&self) -> Arc<Bound<Control>> {
        Arc::new(bind(Control::empty()))
    }

    fn get_subcontroller(&self, _id: &str) -> Option<Arc<Controller>> {
        None
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestController {
        label_controller: Arc<LabelController>,
        view_model: Arc<NullViewModel>
    }
    struct LabelController {
        view_model: Arc<NullViewModel>
    }

    impl TestController {
        pub fn new() -> TestController {
            TestController { 
                label_controller: Arc::new(LabelController::new()), 
                view_model: Arc::new(NullViewModel::new()) 
            }
        }
    }

    impl LabelController {
        pub fn new() -> LabelController {
            LabelController { view_model: Arc::new(NullViewModel::new()) }
        }
    }

    impl Controller for TestController {
        fn ui(&self) -> Arc<Bound<Control>> {
            Arc::new(bind(Control::container().with_controller("Test")))
        }

        fn get_subcontroller(&self, _id: &str) -> Option<Arc<Controller>> {
            Some(self.label_controller.clone())
        }

        fn get_viewmodel(&self) -> Arc<ViewModel> {
            self.view_model.clone()
        }
    }

    impl Controller for LabelController {
        fn ui(&self) -> Arc<Bound<Control>> {
            Arc::new(bind(Control::label()))
        }

        fn get_subcontroller(&self, _id: &str) -> Option<Arc<Controller>> {
            None
        }

        fn get_viewmodel(&self) -> Arc<ViewModel> {
            self.view_model.clone()
        }
    }

    #[test]
    fn can_assemble_simple_label() {
        let label_controller    = Arc::new(LabelController::new());
        let assembly            = assemble_ui(label_controller.clone());

        assert!(assembly.get() == Control::label());
    }

    #[test]
    fn can_assemble_with_subassembly() {
        let test_controller = Arc::new(TestController::new());
        let assembly        = assemble_ui(test_controller.clone());

        assert!(assembly.get() == Control::container()
            .with_controller("Test")
            .with(vec![
                Control::label()
            ]));
    }
}
