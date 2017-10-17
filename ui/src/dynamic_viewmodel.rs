use super::property::*;
use super::viewmodel::*;
use super::binding::*;

use std::sync::*;
use std::collections::HashMap;

///
/// The dynamic viewmodel lets us define arbitrary properties as bound or
/// computed values. A particular key can only be bound or computed: if it
/// is set as both, the computed version 'wins'. 
///
pub struct DynamicViewModel {
    /// Maps bindings in this viewmodel to their values
    bindings: Mutex<HashMap<String, Arc<Binding<PropertyValue>>>>,

    /// Maps computed bindings to their values (we ignore these when setting)
    computed: Mutex<HashMap<String, Arc<Bound<PropertyValue>>>>,

    /// Used for properties that don't exist in this model
    nothing: Arc<Binding<PropertyValue>>
}

impl DynamicViewModel {
    ///
    /// Creates a new dynamic viewmodel
    /// 
    pub fn new() -> DynamicViewModel {
        DynamicViewModel { 
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

    ///
    /// Sets a binding to a computed value 
    ///
    pub fn set_computed<TFn>(&self, property_name: &str, calculate_value: TFn)
    where TFn: 'static+Send+Sync+Fn() -> PropertyValue {
        let new_binding = Arc::new(computed(calculate_value));

        let mut computed = self.computed.lock().unwrap();
        computed.insert(String::from(property_name), new_binding);
    }
}

impl ViewModel for DynamicViewModel {
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
        // The keys for items with 'set' bindings
        let mut binding_keys: Vec<String> = {
            let bindings = self.bindings.lock().unwrap();
            bindings
                .keys()
                .map(|key| key.clone())
                .collect()
        };

        // Keys for items with computed bindings
        let mut computed_keys: Vec<String> = {
            let computed = self.computed.lock().unwrap();
            computed
                .keys()
                .map(|key| key.clone())
                .collect()
        };

        // Combine them and deduplicate for the final list of keys
        binding_keys.append(&mut computed_keys);
        binding_keys.sort();
        binding_keys.dedup();

        binding_keys
    }
}