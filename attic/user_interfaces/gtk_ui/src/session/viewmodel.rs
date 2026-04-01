use super::super::gtk_action::*;

use flo_ui::*;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

///
/// Tracks and generates events for viewmodel changes in GTK
///
pub struct GtkSessionViewModel {
    /// Values set in the viewmodel (controller path to properties)
    values: HashMap<Vec<String>, HashMap<String, PropertyValue>>,

    /// The bindings in this viewmodel
    bindings: HashMap<Vec<String>, HashMap<String, Vec<(WidgetId, Box<dyn Fn(PropertyValue) -> Vec<GtkWidgetAction>>)>>>
}

impl GtkSessionViewModel {
    ///
    /// Creates a new GTK sesion viewmodel, which will send events to the specified sink
    ///
    pub fn new() -> GtkSessionViewModel {
        GtkSessionViewModel {
            values:         HashMap::new(),
            bindings:       HashMap::new()
        }
    }

    ///
    /// Deletes the binding data for a particular widget ID
    ///
    pub fn delete_widget(&mut self, widget_id: WidgetId) {
        // This is a naive algorithm that iterates through the entire set of bindings looking for our widget
        // If there are a large number of bindings, this may be very inefficient
        for (_, property_bindings) in self.bindings.iter_mut() {
            for (_, property_list) in property_bindings.iter_mut() {
                property_list.retain(|&(ref id, _)| *id != widget_id)
            }
        }
    }

    ///
    /// Binds a property to an action to be performed every time it's changed
    ///
    pub fn bind(&mut self, widget_id: WidgetId, controller_path: &Vec<String>, property: &Property, action_fn: Box<dyn Fn(PropertyValue) -> Vec<GtkWidgetAction>>) -> Vec<GtkWidgetAction> {
        match property {
            // Bindings need to be stored for future updates
            &Property::Bind(ref binding) => {
                // Call the action function with the current value of the property to set up the control
                let actions = {
                    let controller_values   = self.values.get(controller_path);
                    let property_value      = controller_values.and_then(|controller_values| controller_values.get(binding));
                    let property_value      = property_value.cloned().unwrap_or(PropertyValue::Nothing);

                    action_fn(property_value)
                };

                // Store the bindings
                let controller_bindings = self.bindings.entry(controller_path.clone()).or_insert_with(|| HashMap::new());
                let property_bindings   = controller_bindings.entry(binding.clone()).or_insert_with(|| vec![]);

                property_bindings.push((widget_id, action_fn));

                // Return any actions that might have been generated for a pre-existing property value
                actions
            },

            // Other properties are fixed values: just run the function immediately
            fixed_value => {
                let maybe_value: Option<PropertyValue>  = fixed_value.clone().into();
                let definitely_value                    = maybe_value.unwrap_or_else(|| PropertyValue::String("<< bad binding >>".to_string()));

                action_fn(definitely_value)
            }
        }
    }

    ///
    /// Update the viewmodel with values from some updates
    ///
    pub fn update(&mut self, updates: Vec<ViewModelUpdate>) -> Vec<GtkAction> {
        let mut actions = vec![];

        for controller_update in updates {
            // Each update is a set of changes to a particular controller
            let property_values = self.values.entry(controller_update.controller_path().clone()).or_insert_with(|| HashMap::new());
            let controller_bindings = self.bindings.get(controller_update.controller_path());

            // Process each update in turn
            for viewmodel_update in controller_update.updates().iter() {
                match viewmodel_update {
                    ViewModelChange::NewProperty(property_name, property_value)     |
                    ViewModelChange::PropertyChanged(property_name, property_value) => {
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
                        if property_changed {
                            // Get the bindings for this property
                            let property_bindings = controller_bindings.and_then(|controller_bindings| controller_bindings.get(property_name));

                            // Push actions for this property into the action list
                            if let Some(property_bindings) = property_bindings {
                                let new_value = property_value.clone();

                                // Generate the actions for each update
                                let update_actions = property_bindings
                                    .iter()
                                    .map(move |&(ref widget_id, ref action)| {
                                        (*widget_id, action(new_value.clone()))
                                    })
                                    .filter(|&(_, ref actions)| actions.len() > 0)
                                    .map(|(widget_id, actions)| GtkAction::Widget(widget_id, actions));

                                // Put into the list of actions to perform
                                actions.extend(update_actions)
                            }
                        }
                    }
                }
            }
        }

        // Result is the actions generated by the property change
        actions
    }
}
