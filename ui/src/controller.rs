use super::binding::*;
use super::control::*;

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
    fn ui(&self) -> Box<Bound<Control>>;

    /// Attempts to retrieve a sub-controller of this controller
    fn get_subcontroller(&self, id: &str) -> Option<Arc<Controller>>;
}

///
/// Returns a bound control that expands the content of any 
/// sub-controllers that might be present.
///
pub fn assemble_ui(base_controller: Arc<Controller>) -> Box<Bound<Control>> {
    // Fetch the UI for the controller
    let base_ui = base_controller.ui();

    // Result is computed
    return Box::new(computed(move || {
        base_ui.get().map(&|control| {
            if let Some(controller_id) = control.controller() {
                // Has a subcontroller
                let subcontroller = base_controller.get_subcontroller(controller_id);

                if let Some(subcontroller) = subcontroller {
                    // If we can look up the subcontroller then this control should have its UI as its subcomponents
                    let subassembly = assemble_ui(subcontroller);
                    control.with(vec![subassembly.get()])
                } else {
                    // No subcontroller
                    control.clone()
                }
            } else {
                // Control is untouched
                control.clone()
            }
        })
    }));
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestController;
    struct LabelController;

    impl Controller for TestController {
        fn ui(&self) -> Box<Bound<Control>> {
            Box::new(bind(Control::container().with_controller("Test")))
        }

        fn get_subcontroller(&self, _id: &str) -> Option<Arc<Controller>> {
            Some(Arc::new(LabelController))
        }
    }

    impl Controller for LabelController {
        fn ui(&self) -> Box<Bound<Control>> {
            Box::new(bind(Control::label()))
        }

        fn get_subcontroller(&self, _id: &str) -> Option<Arc<Controller>> {
            None
        }
    }

    #[test]
    fn can_assemble_simple_label() {
        let label_controller    = Arc::new(LabelController);
        let assembly            = assemble_ui(label_controller);

        assert!(assembly.get() == Control::label());
    }

    #[test]
    fn can_assemble_with_subassembly() {
        let test_controller = Arc::new(TestController);
        let assembly        = assemble_ui(test_controller);

        assert!(assembly.get() == Control::container()
            .with_controller("Test")
            .with(vec![
                Control::label()
            ]));
    }
}
