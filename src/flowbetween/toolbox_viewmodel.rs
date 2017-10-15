use ui::*;

use std::sync::*;
use std::collections::HashMap;

pub struct ToolboxViewModel {
    /// Maps bindings in this viewmodel to their values
    bindings: Mutex<HashMap<String, Arc<Binding<PropertyValue>>>>,

    /// Maps computed bindings to their values (we ignore these when setting)
    computed: Mutex<HashMap<String, Arc<Bound<PropertyValue>>>>,

    /// Used for properties that don't exist in this model
    nothing: Arc<Binding<PropertyValue>>
}

impl ToolboxViewModel {
    ///
    /// Creates a new toolbox viewmodel
    /// 
    pub fn new() -> ToolboxViewModel {
        ToolboxViewModel { 
            bindings:   Mutex::new(HashMap::new()), 
            computed:   Mutex::new(HashMap::new()),
            nothing:    Arc::new(bind(PropertyValue::Nothing)) }
    }

    ///
    /// Attempts to retrieve the set binding with a particular name
    ///
    fn get_binding(&self, property_name: &str) -> Option<Arc<Binding<PropertyValue>>> {
        let bindings = self.bindings.lock().unwrap();

        bindings.get(&String::from(property_name)).map(|arc| arc.clone())
    }

    ///
    /// Attempts to retrieve the computed binding with a paritcular name
    /// 
    fn get_computed(&self, property_name: &str) -> Option<Arc<Bound<PropertyValue>>> {
        let computed = self.computed.lock().unwrap();

        computed.get(&String::from(property_name)).map(|arc| arc.clone())
    }
}

impl ViewModel for ToolboxViewModel {
    fn get_property(&self, property_name: &str) -> Arc<Bound<PropertyValue>> {
        if let Some(result) = self.get_computed(property_name) {
            // Computed values are returned first, so these bindings cannot be set
            result
        } else if let Some(result) = self.get_binding(property_name) {
            // 'Set' bindings are returned next
            result
        } else {
            // If an invalid name is requested, we return something bound to nothing
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