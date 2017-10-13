use super::property::*;

///
/// Represents a viewmodel for a control subtree. ViewModels are
/// used for controls which can be edited and need to have values
/// stored by key in the controller
///
pub trait ViewModel {
    /// Retrieves a property
    fn get_property(&self, property_name: &str) -> Property;

    /// Updates a property
    fn set_property(&mut self, property_name: &str, new_value: Property);

    /// Retrieves the names of all of the properties in this item
    fn get_property_names(&self) -> Vec<String>;
}
