use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::pending_storage_change::*;
use crate::undo::*;
use crate::traits::*;
use crate::storage::*;
use crate::storage::layer_properties::*;

use futures::prelude::*;

use flo_canvas_animation::description::*;

use std::sync::*;
use std::time::{Duration};

impl StreamAnimationCore {
    ///
    /// Performs a layer edit on this animation
    ///
    pub fn layer_edit<'a>(&'a mut self, layer_id: u64, layer_edit: &'a LayerEdit) -> impl 'a+Future<Output=ReversedEdits> {
        use self::LayerEdit::*;

        // Invalidate the layer from the cache
        self.cached_layers.remove(&layer_id);

        // Perform the edit
        async move {
            match layer_edit {
                Paint(when, paint_edit)                                     => { self.paint_edit(layer_id, *when, paint_edit).await }
                Path(when, path_edit)                                       => { self.path_edit(layer_id, *when, path_edit).await }
                CreateAnimation(when, element_id, description)              => { self.create_animation(layer_id, *when, *element_id, description.clone()).await }
                CreateElement(when, element_id, vector)                     => { self.create_element(layer_id, *when, *element_id, vector.clone(), false).await }
                CreateElementUnattachedToFrame(when, element_id, vector)    => { self.create_element(layer_id, *when, *element_id, vector.clone(), true).await }
                AddKeyFrame(when)                                           => { self.add_key_frame(layer_id, *when).await }
                RemoveKeyFrame(when)                                        => { self.remove_key_frame(layer_id, *when).await }
                SetName(new_name)                                           => { self.set_layer_name(layer_id, new_name).await }
                SetOrdering(ordering)                                       => { self.set_layer_ordering(layer_id, *ordering).await }
                SetAlpha(alpha)                                             => { self.set_layer_alpha(layer_id, *alpha).await }
                Cut { path, when, inside_group }   => { 
                    let cut = self.layer_cut(layer_id, *when, Arc::clone(path)).await;
                    self.apply_layer_cut(layer_id, *when, cut, *inside_group).await
                }
            }
        }
    }

    ///
    /// Creates an element with a specific vector definition
    ///
    pub fn create_element<'a>(&'a mut self, layer_id: u64, when: Duration, element_id: ElementId, vector: Vector, unattached: bool) -> impl 'a+Future<Output=ReversedEdits> {
        async move {
            // If the vector won't deserialize properly (which can happen because the vector contains other elements): do nothing as the vector will not show up in the next edit log
            if vector.requires_resolution_for_deserialize() {
                return ReversedEdits::empty();
            }

            // Ensure that the appropriate keyframe is in the cache. No edit can take place if there's no keyframe at this time
            let current_keyframe = match self.edit_keyframe(layer_id, when).await {
                None            => { return ReversedEdits::empty(); }
                Some(keyframe)  => keyframe
            };

            // Make sure the element ID is in sync (the wrapper is what actually stores the ID so this doesn't matter too much right now but this could be important in the future)
            let mut vector = vector;
            vector.set_id(element_id);

            // Create the element wrapper
            let storage_updates = current_keyframe.future_sync(move |current_keyframe| {
                async move {
                    if let Some(existing_wrapper) = current_keyframe.elements.get_mut(&element_id) {
                        // Update the definition of an existing element
                        existing_wrapper.element = vector;

                        let mut changes = PendingStorageChange::new();
                        changes.push(StorageCommand::WriteElement(element_id.id().unwrap(), existing_wrapper.serialize_to_string()));

                        changes
                    } else {
                        // Create a new element
                        let wrapper     = if unattached {
                            ElementWrapper::unattached_with_element(vector, when)
                        } else {
                            ElementWrapper::attached_with_element(vector, when)
                        };
                        let add_element = current_keyframe.add_element_to_end(element_id, wrapper);

                        add_element
                    }
                }.boxed()
            }).await;

            // Send to the storage
            self.request(storage_updates.unwrap()).await;

            // Reversal is just to delete the element
            ReversedEdits::with_edit(AnimationEdit::Element(vec![element_id], ElementEdit::Delete))
        }
    }

    ///
    /// Creates an animation element on a keyframe
    ///
    pub fn create_animation<'a>(&'a mut self, layer_id: u64, when: Duration, element_id: ElementId, description: RegionDescription) -> impl 'a+Future<Output=ReversedEdits> {
        async move {
            // Ensure that the appropriate keyframe is in the cache. No edit can take place if there's no keyframe at this time
            let current_keyframe = match self.edit_keyframe(layer_id, when).await {
                None            => { return ReversedEdits::empty(); }
                Some(keyframe)  => keyframe
            };

            // Create the animation element
            let element = AnimationElement::new(element_id, description);
            let element = Vector::AnimationRegion(element);

            // Edit the keyframe
            let storage_updates = current_keyframe.future_sync(move |current_keyframe| {
                async move {
                    // Add to a wrapper
                    let wrapper     = ElementWrapper::attached_with_element(element, when);

                    // Append to the current keyframe and return the list of storage commands
                    let add_element = current_keyframe.add_element_to_end(element_id, wrapper);

                    add_element
                }.boxed()
            }).await;

            // Send to the storage
            self.request(storage_updates.unwrap()).await;

            ReversedEdits::with_edit(AnimationEdit::Element(vec![element_id], ElementEdit::Delete))
        }
    }

    ///
    /// Sets the order of a layer (which is effectively the ID of the layer this layer should appear behind)
    ///
    pub fn set_layer_ordering<'a>(&'a mut self, layer_id: u64, order_behind: u64) -> impl 'a+Future<Output=ReversedEdits> {
        async move {
            // A layer is always ordered behind itself
            if order_behind == layer_id {
                return ReversedEdits::empty();
            }

            // Read all of the layers from storage
            let layers          = self.request(vec![StorageCommand::ReadLayers]).await;
            let mut layers      = layers.unwrap_or_else(|| vec![]).into_iter().map(|response| {
                    if let StorageResponse::LayerProperties(layer_id, properties) = response {
                        let properties = LayerProperties::deserialize(&mut properties.chars()).unwrap_or_else(|| LayerProperties::default());
                        Some((layer_id, properties))
                    } else {
                        None
                    }
                })
                .flatten()
                .collect::<Vec<_>>();

            // Sort the layers into order
            layers.sort_by(|(id_a, layer_a), (id_b, layer_b)| {
                if layer_a.ordering == layer_b.ordering {
                    id_a.cmp(&id_b)
                } else {
                    layer_a.ordering.cmp(&layer_b.ordering)
                }
            });

            // Find the layer and the layer we need to order behind
            let layer_index         = layers.iter().enumerate().filter(|(_, (id, _))| {
                    *id == layer_id
                })
                .map(|(index, _)| index)
                .nth(0);
            let order_behind_index  = layers.iter().enumerate().filter(|(_, (id, _))| {
                    *id == order_behind
                })
                .map(|(index, _)| index)
                .nth(0);

            let (layer_index, order_behind_index) = match (layer_index, order_behind_index) {
                (Some(layer_index), Some(order_behind_index))   => (layer_index, order_behind_index),
                _                                               => { return ReversedEdits::empty(); }
            };

            // Do nothing if the layer is being ordered back to where it already is
            if layer_index == order_behind_index || order_behind_index == layer_index + 1 {
                return ReversedEdits::empty();
            }

            // The reverse is to move the layer back behind whatever layer it is already behind
            let reversed_edits = if layer_index < layers.len() - 2 {
                // Move the layer behind the layer it's already behind
                let originally_behind_id = layers[layer_index+1].0;
                ReversedEdits::with_edit(AnimationEdit::Layer(layer_id, LayerEdit::SetOrdering(originally_behind_id)))
            } else {
                // Move the layer to the front (we can't actually move to the front in one operation, so we move behind the frontmost layer and then move it back behind it)
                let frontmost_id = layers[layers.len()-2].0;
                ReversedEdits::with_edits(vec![
                    AnimationEdit::Layer(layer_id, LayerEdit::SetOrdering(frontmost_id)),
                    AnimationEdit::Layer(frontmost_id, LayerEdit::SetOrdering(layer_id))
                ])
            };

            // Move the layer behind the 'order-behind' index
            let (_, layer_props) = layers.remove(layer_index);
            if order_behind_index > layer_index {
                layers.insert(order_behind_index-1, (layer_id, layer_props));
            } else {
                layers.insert(order_behind_index, (layer_id, layer_props));
            }

            // Update the layer ordering
            for layer_num in 0..layers.len() {
                layers[layer_num].1.ordering = layer_num as i64;
            }

            // Save all of the layers
            self.request(layers.into_iter()
                    .map(|(layer_id, layer_properties)| {
                        let mut serialized = String::new();
                        layer_properties.serialize(&mut serialized);

                        (layer_id, serialized)
                    })
                    .map(|(layer_id, serialized)| StorageCommand::WriteLayerProperties(layer_id, serialized)))
                .await;

            reversed_edits
        } 
    }

    ///
    /// Adds a new layer with a particular ID to this animation
    ///
    pub fn add_new_layer<'a>(&'a mut self, layer_id: u64) -> impl 'a+Future<Output=ReversedEdits> {
        async move {
            // Fetch the existing layers to discover the maximum ordering ID
            let existing_layers = self.storage_connection.read_all_layer_properties().await;
            let max_ordering    = existing_layers.iter()
                .flat_map(|(_id, layer)| {
                    if layer.ordering == i64::max_value() {
                        None
                    } else {
                        Some(layer.ordering)
                    }
                })
                .reduce(|a, b| i64::max(a, b));

            // Create the default properties for this layer
            let mut properties  = LayerProperties::default();
            properties.ordering = max_ordering.unwrap_or(0) + 1;
            let mut serialized  = String::new();
            properties.serialize(&mut serialized);

            // Add the layer
            self.request_one(StorageCommand::AddLayer(layer_id, serialized)).await;

            ReversedEdits::with_edit(AnimationEdit::RemoveLayer(layer_id))
        }
    }

    ///
    /// Removes the layer with the specified ID from the animation
    ///
    pub fn remove_layer<'a>(&'a mut self, layer_id: u64) -> impl 'a+Future<Output=ReversedEdits> {
        async move {
            // Generate the commands to recreate the layer
            let reverse = ReversedEdits::with_recreated_layer(layer_id, &mut self.storage_connection).await;

            // Remove the layer
            self.request_one(StorageCommand::DeleteLayer(layer_id)).await;

            reverse
        }
    }

    ///
    /// Sets the name of a layer
    ///
    pub fn set_layer_name<'a>(&'a mut self, layer_id: u64, name: &'a str) -> impl 'a+Future<Output=ReversedEdits> { 
        async move {
            // Read the current properties for this layer
            let mut properties = match self.request_one(StorageCommand::ReadLayerProperties(layer_id)).await {
                Some(StorageResponse::LayerProperties(_, properties)) => {
                    LayerProperties::deserialize(&mut properties.chars())
                        .unwrap_or_else(|| LayerProperties::default())
                }

                _ => LayerProperties::default()
            };

            // Update the name
            let old_name    = properties.name;
            properties.name = name.to_string();

            // Save back to the storage
            let mut serialized = String::new();
            properties.serialize(&mut serialized);
            self.request_one(StorageCommand::WriteLayerProperties(layer_id, serialized)).await;

            ReversedEdits::with_edit(AnimationEdit::Layer(layer_id, LayerEdit::SetName(old_name)))
        } 
    }

    ///
    /// Sets the alpha blend factor of a layer
    ///
    pub fn set_layer_alpha<'a>(&'a mut self, layer_id: u64, alpha: f64) -> impl 'a+Future<Output=ReversedEdits> { 
        async move {
            // Read the current properties for this layer
            let mut properties = match self.request_one(StorageCommand::ReadLayerProperties(layer_id)).await {
                Some(StorageResponse::LayerProperties(_, properties)) => {
                    LayerProperties::deserialize(&mut properties.chars())
                        .unwrap_or_else(|| LayerProperties::default())
                }

                _ => LayerProperties::default()
            };

            // Update the alpha value
            let old_alpha       = properties.alpha;
            properties.alpha    = f64::min(1.0, f64::max(0.0, alpha));

            // Save back to the storage
            let mut serialized = String::new();
            properties.serialize(&mut serialized);
            self.request_one(StorageCommand::WriteLayerProperties(layer_id, serialized)).await;

            ReversedEdits::with_edit(AnimationEdit::Layer(layer_id, LayerEdit::SetAlpha(old_alpha)))
        } 
    }
}
