use super::binding::*;
use super::control::*;

use std::ops::Deref;
use std::any::Any;

///
/// The generic controller is used to hide the types of sub-controllers
///
pub type GenericController = Controller<ControllerSpecifier=Box<Any>>;

///
/// Controllers represent a portion of the UI and provide a hub for
/// receiving events related to it and connecting the model or
/// viewmodel.
///
/// The subcontroller specifier can be used to retrieve controllers
/// that deal with a different part of the UI. When passing events
/// and IDs between controllers, we use strings (JSON, serialized
/// and deserialized by serde). 
///
pub trait Controller {
    type ControllerSpecifier;
    
    /// Retrieves a Control representing the UI for this controller
    fn ui(&self) -> Box<Bound<Control>>;

    /// Attempts to retrieve a sub-controller of this controller
    fn get_subcontroller(&self, id: &Self::ControllerSpecifier) -> Option<Box<GenericController>>;
}

///
/// Provides the generic controller interface for any controller
/// along with a deref implementation so the 'native' interface
/// is also available.
///
pub struct AnyController<TController: Controller>(TController);

impl<TController: Controller> AnyController<TController> {
    ///
    /// Creates a new controller whose identifiers can be represented generically
    ///
    pub fn from(controller: TController) -> AnyController<TController> {
        AnyController(controller)
    }
}

impl<TController: Controller> Controller for AnyController<TController>
where TController::ControllerSpecifier: 'static {
    type ControllerSpecifier = Box<Any>;

    fn ui(&self) -> Box<Bound<Control>> {
        // UI is just passed straight through
        let real_controller = &self.0;

        real_controller.ui()
    }

    fn get_subcontroller(&self, id: &Box<Any>) -> Option<Box<GenericController>> {
        if let Some(real_id) = id.downcast_ref::<TController::ControllerSpecifier>() {
            // Valid IDs are passed through
            let AnyController(ref real_controller) = *self;
            real_controller.get_subcontroller(real_id)
        } else {
            // Invalid IDs just produce no controller
            None
        }
    }
}

impl<TController: Controller> Deref for AnyController<TController> {
    type Target = TController;

    fn deref(&self) -> &TController {
        &self.0   
    }
}

///
/// The null controller is the base controller, which controls nothing
///
pub struct NullController;

impl NullController {
    pub fn new() -> NullController {
        NullController
    }
}

impl Controller for NullController {
    type ControllerSpecifier = ();

    fn ui(&self) -> Box<Bound<Control>> {
        Box::new(bind(Control::empty()))
    }

    fn get_subcontroller(&self, _id: &Self::ControllerSpecifier) -> Option<Box<GenericController>> {
        None
    }
}
