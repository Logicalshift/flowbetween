use ui::*;

use std::sync::*;

///
/// Describes an update to the view model
///
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewModelUpdate {
    /// The controller that owns the view model being updated
    controller_path: Vec<String>,

    /// Key/value pairs of the updates to the view model for this controller
    updates: Vec<(String, PropertyValue)>
}

impl ViewModelUpdate {
    ///
    /// Creates a new view model update
    ///
    pub fn new(controller_path: Vec<String>, updates: Vec<(String, PropertyValue)>) -> ViewModelUpdate {
        ViewModelUpdate {
            controller_path:    controller_path,
            updates:            updates
        }
    }

    ///
    /// Returns the path to the controller that owns this view model
    ///
    pub fn controller_path(&self) -> &Vec<String> {
        &self.controller_path
    }

    ///
    /// Returns the changes that have been made to this view model
    ///
    pub fn updates(&self) -> &Vec<(String, PropertyValue)> {
        &self.updates
    }
}

///
/// Returns an update for all of the keys in a particular viewmodel
///
pub fn viewmodel_update_all(controller_path: Vec<String>, viewmodel: &ViewModel) -> ViewModelUpdate {
    let keys        = viewmodel.get_property_names();
    let mut updates = vec![];

    for property_name in keys.iter() {
        let value = viewmodel.get_property(&*property_name);
        updates.push(((*property_name).clone(), value.get()));
    }

    return ViewModelUpdate::new(controller_path, updates);
}

///
/// Generates the updates to set the viewmodel for an entire controller tree
///
pub fn viewmodel_update_controller_tree(controller: &Arc<Controller>) -> Vec<ViewModelUpdate> {
    let mut result = vec![];

    // Push the controllers to the result
    // Rust could probably capture the 'result' variable in the closure exactly liek this if it were smarter
    fn add_controller_to_result(controller: &Arc<Controller>, path: &mut Vec<String>, result: &mut Vec<ViewModelUpdate>) {
        // Fetch the update for the viewmodel for this controller
        let viewmodel           = controller.get_viewmodel();
        let viewmodel_update    = viewmodel_update_all(path.clone(), &*viewmodel);

        // Add to the result if there are any entries in this viewmodel
        if viewmodel_update.updates().len() > 0 {
            result.push(viewmodel_update);
        }

        // Visit any subcontrollers found in this controllers UI
        let controller_ui   = controller.ui().get();
        let subcontrollers  = controller_ui.all_controllers();

        for subcontroller_name in subcontrollers.iter() {
            if let Some(subcontroller) = controller.get_subcontroller(subcontroller_name) {
                // Recursively process this subcontroller
                path.push(subcontroller_name.clone());
                add_controller_to_result(&subcontroller, path, result);
                path.pop();
            }
        }
    }

    // Recursively add the controllers starting at the current one
    add_controller_to_result(controller, &mut vec![], &mut result);

    result
}
