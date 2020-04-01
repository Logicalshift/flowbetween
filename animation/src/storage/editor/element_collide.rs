use super::keyframe_core::*;
use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::super::storage_api::*;
use super::super::super::traits::*;

use futures::prelude::*;
use ::desync::*;

use std::sync::*;

impl StreamAnimationCore {
    ///
    /// Discovers all of the elements in the frame along with their properties
    ///
    fn frame_elements_with_properties(frame: Arc<Desync<KeyFrameCore>>) -> impl Send+Future<Output=Vec<(ElementWrapper, Arc<VectorProperties>)>> {
        async move {
            frame.future(move |frame| {
                async move {
                    // Start with the default properties
                    let mut current_properties  = Arc::new(VectorProperties::default());
                    let mut result              = vec![];

                    // If this is a vector frame, apply the properties from each element
                    let mut next_element_id = frame.initial_element;
                    while let Some(current_element_id) = next_element_id {
                        // Fetch the element from the frame
                        let current_element = frame.elements.get(&current_element_id);

                        if let Some(elem) = current_element {
                            // Update the properties for this element
                            current_properties = frame.apply_properties_for_element(&elem.element, current_properties);

                            // Add to the result
                            result.push((elem.clone(), Arc::clone(&current_properties)));
                        }

                        // Move on to the element that's ahead of the current one
                        next_element_id = current_element.and_then(|element| element.order_before);
                    }

                    result
                }.boxed()
            }).await.unwrap()
        }
    }

    ///
    /// Attempts to combine an element with other elements in the same frame (by joining them into a single path)
    ///
    pub fn collide_with_existing_elements<'a>(&'a mut self, combine_element_id: ElementId) -> impl 'a+Send+Future<Output=()> {
        async move {
            // Fetch the frame that this element belongs to
            let assigned_element_id = match combine_element_id.id() {
                Some(id)    => id,
                None        => { return }
            };

            if let Some(frame) = self.edit_keyframe_for_element(assigned_element_id).await {
                // We need to know the properties of all of the elements in the current frame (we need to work backwards to generate the grouped element)
                let elements_with_properties    = Self::frame_elements_with_properties(Arc::clone(&frame)).await;

                // Nothing to do if there are no properties
                if elements_with_properties.len() == 0 {
                    return;
                }

                let updates = frame.future(move |frame| {
                    async move {
                        // Find the brush properties for the selected element. These are usually at the end, so a linear search like this should be fine
                        let new_properties = elements_with_properties.iter().rev()
                            .filter(|elem| elem.0.element.id() == combine_element_id)
                            .map(|elem| elem.1.clone())
                            .nth(0)
                            .unwrap_or_else(|| Arc::new(VectorProperties::default()));
                        let current_brush = &new_properties.brush;

                        // Fetch the element from the frame
                        let wrapper     = frame.elements.get(&combine_element_id);
                        let mut updates = vec![];

                        let wrapper     = match wrapper {
                            Some(wrapper)   => wrapper,
                            None            => { return updates; }
                        };

                        // Collide other elements in the frame with this element
                        // Only brush stroke elements can be combined at the moment
                        match &wrapper.element {
                            Vector::BrushStroke(brush_stroke) => {
                                // Take the brush points from this one to generate a new value for the element
                                let brush_points = brush_stroke.points();

                                // Attempt to combine the element we fetched with the rest of the frame
                                let mut combined_element            = None;
                                for (combine_with_wrapper, properties) in elements_with_properties.iter().rev() {
                                    use self::CombineResult::*;

                                    // Ignore the element we're merging
                                    // TODO: consider ignoring the elements above the element we're merging too
                                    if combine_with_wrapper.element.id() == combine_element_id {
                                        continue;
                                    }

                                    let new_combined = match current_brush.combine_with(&combine_with_wrapper.element, Arc::clone(&brush_points), &new_properties, &*properties, combined_element.clone()) {
                                        NewElement(new_combined)    => {
                                            // Unlink the element from the frame (brushes typicaly put their new element into a group so
                                            // this will set up the element in a way that's appropriate for that)
                                            updates.extend(frame.unlink_element(combine_with_wrapper.element.id()));

                                            Some(new_combined) 
                                        },

                                        NoOverlap                   => { continue; },               // Might be able to combine with an element further down
                                        CannotCombineAndOverlaps    => { break; },                  // Not quite right: we can combine with any element that's not obscured by an existing element (we can skip over overlapping elements we can't combine with)
                                        UnableToCombineFurther      => { break; }                   // Always stop here
                                    };

                                    combined_element = new_combined;
                                }

                                // Final update is to replace the old element with the new element
                                let replacement_element = frame.elements.get(&combine_element_id).cloned();
                                if let (Some(combined_element), Some(mut replacement_element)) = (combined_element, replacement_element) {
                                    // Replace the element
                                    replacement_element.element = combined_element;

                                    // Update it in the storage
                                    updates.push(StorageCommand::WriteElement(assigned_element_id, replacement_element.serialize_to_string()));
                                    frame.elements.insert(combine_element_id, replacement_element);
                                } else {
                                    // If nothing was generated then any updates that might have been generated are not valid
                                    updates = vec![];
                                }
                            }

                            _ => { }
                        }

                        // The result is the list of updates we want to perform
                        updates
                    }.boxed()
                }).await.unwrap();
                
                // Send the updates to storage
                self.request(updates).await;
            }
        }
    }
}
