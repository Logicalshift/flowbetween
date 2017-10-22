use super::property::*;

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

    ///
    /// Adds a new controller name to the start of this path 
    ///
    pub fn add_to_start_of_path(&mut self, new_controller: String) {
        self.controller_path.insert(0, new_controller);
    }
}
