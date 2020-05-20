use super::element_wrapper::*;
use super::stream_animation_core::*;
use crate::storage::storage_api::*;
use crate::traits::*;

use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};

impl StreamAnimationCore {
    ///
    /// Performs a path edit on a layer
    ///
    pub fn path_edit<'a>(&'a mut self, layer_id: u64, when: Duration, edit: &'a PathEdit) -> impl 'a+Future<Output=()> {
        async move {
            use self::PathEdit::*;

            // Ensure that the appropriate keyframe is in the cache. No edit can take place if there's no keyframe at this time
            let current_keyframe = match self.edit_keyframe(layer_id, when).await {
                None            => { return; }
                Some(keyframe)  => keyframe
            };

            match edit {
                CreatePath(element_id, components)      => {
                    let element_id = *element_id;

                    // Need to have the brush definition and properties defined for the current path
                    let (defn, props) = if let (Some(defn), Some(props)) = (&self.path_brush_defn, &self.path_brush_props) {
                        (defn.clone(), props.clone())
                    } else {
                        // No properties set
                        return;
                    };

                    // Create the path element
                    let element = PathElement::new(element_id, Path::from_elements(components.iter().cloned()), defn.clone(), props.clone());
                    let element = Vector::Path(element);

                    // Edit the keyframe
                    let storage_updates = current_keyframe.future(move |current_keyframe| {
                        async move {
                            // Add to a wrapper
                            let wrapper         = ElementWrapper::attached_with_element(element, when);

                            // Append to the current keyframe and return the list of storage commands
                            let mut add_element = current_keyframe.add_element_to_end(element_id, wrapper);

                            // Make sure the definition and properties are attached to the keyframe so the path can find them later on
                            add_element.push(StorageCommand::AttachElementToLayer(layer_id, defn.id().id().unwrap_or(0), when));
                            add_element.push(StorageCommand::AttachElementToLayer(layer_id, props.id().id().unwrap_or(0), when));

                            add_element
                        }.boxed()
                    }).await;

                    // Send to the storage
                    self.request(storage_updates.unwrap()).await;
                }

                SelectBrush(element_id, defn, style)    => {
                    // Create a brush definition element
                    let defn                    = BrushDefinitionElement::new(*element_id, defn.clone(), *style);
                    self.path_brush_defn        = Some(Arc::new(defn.clone()));

                    // Save as an element (it gets attached to a frame when used in a path)
                    let element                 = Vector::BrushDefinition(defn);
                    let element_id              = element_id.id().unwrap_or(0);
                    let element_wrapper         = ElementWrapper::unattached_with_element(element, when);

                    let mut element_string  = String::new();
                    element_wrapper.serialize(&mut element_string);

                    self.request(vec![StorageCommand::WriteElement(element_id, element_string)]).await;
                }

                BrushProperties(element_id, properties) => {
                    // Create a brush properties element
                    let defn                    = BrushPropertiesElement::new(*element_id, properties.clone());
                    self.path_brush_props       = Some(Arc::new(defn.clone()));

                    // Save as an element
                    let element                 = Vector::BrushProperties(defn);
                    let element_id              = element_id.id().unwrap_or(0);
                    let element_wrapper         = ElementWrapper::unattached_with_element(element, when);

                    let mut element_string  = String::new();
                    element_wrapper.serialize(&mut element_string);

                    self.request(vec![StorageCommand::WriteElement(element_id, element_string)]).await;
                }
            };
        }
    }
}
