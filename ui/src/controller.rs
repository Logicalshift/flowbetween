use super::binding::*;
use super::control::*;

use std::ops::Deref;

use serde::*;
use serde_json::*;

///
/// The generic controller is used to hide the types of sub-controllers
///
type GenericController = Controller<SubControllerSpecifier=String>;

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
    type SubControllerSpecifier;
    
    /// Retrieves a Control representing the UI for this controller
    fn ui(&self) -> Box<Bound<Control>>;

    /// Attempts to retrieve a sub-controller of this controller
    fn get_subcontroller(&self, id: Self::SubControllerSpecifier) -> Option<Box<GenericController>>;
}

///
/// Provides a controller interface where all the identifiers
/// are JSON strings.
///
pub struct JsonController<TController: Controller>(TController);

impl<TController: Controller> JsonController<TController>
where for<'de> TController::SubControllerSpecifier: Serialize+Deserialize<'de> {
    ///
    /// Creates a new controller whose identifiers are strings
    ///
    pub fn from(controller: TController) -> JsonController<TController> {
        JsonController(controller)
    }
}

impl<TController: Controller> Controller for JsonController<TController>
where for<'de> TController::SubControllerSpecifier: Serialize+Deserialize<'de> {
    type SubControllerSpecifier = String;

    fn ui(&self) -> Box<Bound<Control>> {
        // UI is just passed straight through
        let real_controller = &self.0;

        real_controller.ui()
    }

    fn get_subcontroller(&self, id: String) -> Option<Box<GenericController>> {
        // Need to deserialize the real ID and pass it through
        let real_id = from_str::<TController::SubControllerSpecifier>(&id);

        if let Ok(real_id) = real_id {
            // Valid IDs are passed through
            let JsonController(ref real_controller) = *self;
            real_controller.get_subcontroller(real_id)
        } else {
            // Invalid IDs just produce no controller
            None
        }
    }
}

impl<TController: Controller> Deref for JsonController<TController> {
    type Target = TController;

    fn deref(&self) -> &TController {
        &self.0   
    }
}
