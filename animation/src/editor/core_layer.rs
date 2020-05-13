use super::stream_animation_core::*;
use crate::traits::*;
use crate::storage::storage_api::*;
use crate::storage::layer_properties::*;

use futures::prelude::*;

impl StreamAnimationCore {
    ///
    /// Performs a layer edit on this animation
    ///
    pub fn layer_edit<'a>(&'a mut self, layer_id: u64, layer_edit: &'a LayerEdit) -> impl 'a+Future<Output=()> {
        use self::LayerEdit::*;

        async move {
            match layer_edit {
                Paint(when, paint_edit)     => { self.paint_edit(layer_id, *when, paint_edit).await }
                Path(when, path_edit)       => { self.path_edit(layer_id, *when, path_edit).await }
                AddKeyFrame(when)           => { self.add_key_frame(layer_id, *when).await }
                RemoveKeyFrame(when)        => { self.remove_key_frame(layer_id, *when).await }
                SetName(new_name)           => { self.set_layer_name(layer_id, new_name).await }
                SetOrdering(ordering)       => { self.set_layer_ordering(layer_id, *ordering).await }
            }
        }
    }

    ///
    /// Sets the order of a layer (which is effectively the ID of the layer this layer should appear behind)
    ///
    pub fn set_layer_ordering<'a>(&'a mut self, layer_id: u64, order_behind: u64) -> impl 'a+Future<Output=()> {
        async move {
            // A layer is always ordered behind itself
            if order_behind == layer_id {
                return;
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
            layers.sort_by(|(_, layer_a), (_, layer_b)| {
                layer_a.ordering.cmp(&layer_b.ordering)
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
                _                                               => { return; }
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
        } 
    }

    ///
    /// Adds a new layer with a particular ID to this animation
    ///
    pub fn add_new_layer<'a>(&'a mut self, layer_id: u64) -> impl 'a+Future<Output=()> {
        async move {
            // Create the default properties for this layer
            let properties      = LayerProperties::default();
            let mut serialized  = String::new();
            properties.serialize(&mut serialized);

            // Add the layer
            self.request_one(StorageCommand::AddLayer(layer_id, serialized)).await;
        }
    }

    ///
    /// Removes the layer with the specified ID from the animation
    ///
    pub fn remove_layer<'a>(&'a mut self, layer_id: u64) -> impl 'a+Future<Output=()> {
        async move {
            // Remove the layer
            self.request_one(StorageCommand::DeleteLayer(layer_id)).await;
        }
    }

    ///
    /// Sets the name of a layer
    ///
    pub fn set_layer_name<'a>(&'a mut self, layer_id: u64, name: &'a str) -> impl 'a+Future<Output=()> { 
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
            properties.name = name.to_string();

            // Save back to the storage
            let mut serialized = String::new();
            properties.serialize(&mut serialized);
            self.request_one(StorageCommand::WriteLayerProperties(layer_id, serialized)).await;
        } 
    }
}
