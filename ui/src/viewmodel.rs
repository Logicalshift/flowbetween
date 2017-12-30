use binding::*;

use super::property::*;

///
/// Represents a viewmodel for a control subtree. ViewModels are
/// used for controls which can be edited and need to have values
/// stored by key in the controller
///
pub trait ViewModel {
    /// Retrieves a property
    fn get_property(&self, property_name: &str) -> BindRef<PropertyValue>;

    /// Updates a property
    fn set_property(&self, property_name: &str, new_value: PropertyValue);

    /// Retrieves the names of all of the properties in this item
    fn get_property_names(&self) -> Vec<String>;
}

pub struct NullViewModel {
    nothing: BindRef<PropertyValue>
}

impl NullViewModel {
    pub fn new() -> NullViewModel {
        NullViewModel { nothing: BindRef::from(bind(PropertyValue::Nothing)) }
    }
}

impl ViewModel for NullViewModel {
    fn get_property(&self, _property_name: &str) -> BindRef<PropertyValue> {
        self.nothing.clone()
    }

    fn set_property(&self, _property_name: &str, _new_value: PropertyValue) { 
    }

    fn get_property_names(&self) -> Vec<String> {
        vec![]
    }
}
