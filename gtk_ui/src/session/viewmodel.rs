use super::super::gtk_action::*;

use flo_ui::*;

use futures::*;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

///
/// Tracks and generates events for viewmodel changes in GTK
/// 
pub struct GtkSessionViewModel {
    /// The sink where the actions for this viewmodel should go
    action_sink: Box<Sink<SinkItem=Vec<GtkAction>, SinkError=()>>,

    /// Values set in the viewmodel (controller path to properties)
    values: HashMap<Vec<String>, HashMap<String, PropertyValue>>
}

impl GtkSessionViewModel {
    ///
    /// Creates a new GTK sesion viewmodel, which will send events to the specified sink
    /// 
    pub fn new(action_sink: Box<Sink<SinkItem=Vec<GtkAction>, SinkError=()>> ) -> GtkSessionViewModel {
        GtkSessionViewModel {
            action_sink:    action_sink,
            values:         HashMap::new()
        }
    }

    ///
    /// Update the viewmodel with values from some updates
    /// 
    pub fn update(&mut self, updates: Vec<ViewModelUpdate>) {
        for controller_update in updates {
            // Each update is a set of changes to a particular controller
            let mut property_values = self.values.entry(controller_update.controller_path().clone()).or_insert_with(|| HashMap::new());

            // Process each update in turn
            for &(ref property_name, ref property_value) in controller_update.updates() {
                // Update the property in the model
                let property_changed = {
                    match property_values.entry(property_name.clone()) {
                        Entry::Occupied(mut occupied)  => {
                            if occupied.get() == property_value {
                                // Entry exists but is unchanged
                                false
                            } else {
                                // Entry exists and is changed
                                *occupied.get_mut() = property_value.clone();
                                true
                            }
                        },

                        Entry::Vacant(vacant) => {
                            // Create a new entry
                            vacant.insert(property_value.clone());
                            true
                        }
                    }
                };

                // If the property is changed, generate the events to send to the GTK sink
            }
        }
    }
}
