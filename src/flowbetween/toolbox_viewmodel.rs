use ui::*;

use std::sync::*;
use std::collections::HashMap;

pub struct ToolboxViewModel {
    /// Maps bindings in this viewmodel to their values
    bindings: Mutex<HashMap<String, Arc<Binding<PropertyValue>>>>,

    /// Used for properties that don't exist in this model
    nothing: Arc<Binding<PropertyValue>>
}

impl ToolboxViewModel {
    ///
    /// Creates a new toolbox viewmodel
    /// 
    pub fn new() -> ToolboxViewModel {
        ToolboxViewModel { 
            bindings: Mutex::new(HashMap::new()), 
            nothing: Arc::new(bind(PropertyValue::Nothing)) }
    }
}

impl ViewModel for ToolboxViewModel {
    fn get_property(&self, property_name: &str) -> Arc<Bound<PropertyValue>> {
        let bindings = self.bindings.lock().unwrap();

        if let Some(value) = bindings.get(&String::from(property_name)) {
            value.clone()
        } else {
            self.nothing.clone()
        }
    }

    fn set_property(&self, property_name: &str, new_value: PropertyValue) { 
        let mut bindings = self.bindings.lock().unwrap();

        if let Some(value) = bindings.get(&String::from(property_name)) {
            // Trick here is that while the bindings aren't mutable, their clones can be (and refer to the same place)
            (**value).clone().set(new_value);

            // Awkward return because rust keeps the borrow in the else clause even though nothing can reference it
            return;
        }

        // Property does not exist in this viewmodel: create a new one
        let new_binding = bind(new_value);
        bindings.insert(String::from(property_name), Arc::new(new_binding));
    }

    fn get_property_names(&self) -> Vec<String> {
        let bindings        = self.bindings.lock().unwrap();
        let binding_keys    = bindings
            .keys()
            .map(|key| key.clone())
            .collect();

        binding_keys
    }
}