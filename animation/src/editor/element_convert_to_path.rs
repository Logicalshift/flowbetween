use super::stream_animation_core::*;
use crate::storage::storage_api::*;
use crate::traits::*;

use futures::prelude::*;

use std::sync::*;

impl StreamAnimationCore {
    ///
    /// Converts a particular element from its current type to a path element
    ///
    pub fn convert_element_to_path<'a>(&'a mut self, convert_element_id: ElementId) -> impl 'a+Send+Future<Output=()> {
        async move {
            // Fetch the frame that the element is in
            // 
            // We need the properties associated with the element to convert it to a path; currently the only way to
            // fetch them is with the frame (we could also add a similar method that fetches the element and its
            // properties only but we also need to update the cached frame storage once we're done so we'd end up
            // fetching the frame anyway)
            let assigned_element_id = match convert_element_id.id() {
                Some(id)        => id,
                None            => { return; }
            };

            let frame = match self.edit_keyframe_for_element(assigned_element_id).await {
                Some(frame)     => frame,
                None            => { return; }
            };

            let updates = frame.future(move |frame| {
                async move {
                    // Fetch the element from the frame
                    let mut wrapper = match frame.elements.get(&convert_element_id) {
                        Some(wrapper)   => wrapper.clone(),
                        None            => { return vec![]; }
                    };

                    // Create the vector properties by applying all the attachments for the element
                    let mut vector_properties   = Arc::new(VectorProperties::default());
                    let mut brush_definition    = BrushDefinitionElement::default();
                    let mut brush_properties    = BrushPropertiesElement::default();

                    for attachment_id in wrapper.attachments.iter() {
                        if let Some(attachment) = frame.elements.get(attachment_id) {
                            // Apply the properties from this
                            vector_properties = attachment.element.update_properties(vector_properties, frame.start);

                            // Capture brush definition & properties elements
                            match &attachment.element {
                                Vector::BrushDefinition(brush_defn)     => { brush_definition = brush_defn.clone(); },
                                Vector::BrushProperties(brush_props)    => { brush_properties = brush_props.clone(); },
                                _                                       => { }
                            }
                        }
                    }

                    // Convert the element to a path
                    let path        = wrapper.element.to_path(&*vector_properties, PathConversion::RemoveInteriorPoints);
                    let path        = path.unwrap_or(vec![]).into_iter();
                    let path        = path.filter(|path| path.elements().count() > 2);
                    let path        = path.map(|path| path.elements().collect::<Vec<_>>()).flatten();
                    let path        = path.collect::<Vec<_>>();
                    let path        = Path::from_elements(path);

                    let path        = PathElement::new(wrapper.element.id(), path, Arc::new(brush_definition), Arc::new(brush_properties));
                    let path        = Vector::Path(path);

                    // Update the wrapper
                    wrapper.element = path;
                    
                    // Create the updates to send to storage
                    let updates     = vec![StorageCommand::WriteElement(assigned_element_id, wrapper.serialize_to_string())];

                    // Replace the wrapper in the frame
                    frame.elements.insert(convert_element_id, wrapper);

                    updates
                }.boxed()
            }).await.unwrap();

            // Send the updates to storage
            self.request(updates).await;
        }
    }
}
