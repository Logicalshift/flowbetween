use ui::*;

use std::ops::Deref;

use serde::*;
use serde_json::*;

///
/// Provides a controller interface where all the identifiers
/// are JSON strings.
///
pub struct JsonController<TController: Controller>(TController);

impl<TController: Controller> JsonController<TController>
where for<'de> TController::ControllerSpecifier: Serialize+Deserialize<'de> {
    ///
    /// Creates a new controller whose identifiers are strings
    ///
    pub fn from(controller: TController) -> JsonController<TController> {
        JsonController(controller)
    }
}

impl<TController: Controller> Controller for JsonController<TController>
where for<'de> TController::ControllerSpecifier: Serialize+Deserialize<'de> {
    type ControllerSpecifier = String;

    fn ui(&self) -> Box<Bound<Control>> {
        // UI is just passed straight through
        let real_controller = &self.0;

        real_controller.ui()
    }

    fn get_subcontroller(&self, id: &String) -> Option<Box<GenericController>> {
        // Need to deserialize the real ID and pass it through
        let real_id = from_str::<TController::ControllerSpecifier>(&id);

        if let Ok(real_id) = real_id {
            // Valid IDs are passed through
            let JsonController(ref real_controller) = *self;
            real_controller.get_subcontroller(&real_id)
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
