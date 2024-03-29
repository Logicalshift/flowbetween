use super::keyframe_core::*;
use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::pending_storage_change::*;
use crate::undo::*;
use crate::traits::*;

use futures::prelude::*;
use ::desync::*;

use std::sync::*;
use std::time::{Duration};

impl StreamAnimationCore {
    ///
    /// Discovers all of the elements in the frame along with their properties
    ///
    fn frame_elements_with_properties(frame: Arc<Desync<KeyFrameCore>>, when: Duration) -> impl Send+Future<Output=Vec<(ElementWrapper, Arc<VectorProperties>)>> {
        async move {
            frame.future_sync(move |frame| {
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
                            current_properties = frame.apply_properties_for_element(&elem.element, current_properties, when);

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
    pub fn collide_with_existing_elements<'a>(&'a mut self, source_element_id: ElementId) -> impl 'a+Send+Future<Output=ReversedEdits> {
        async move {
            // Fetch the frame that this element belongs to
            let assigned_element_id = match source_element_id.id() {
                Some(id)    => id,
                None        => { return ReversedEdits::empty(); }
            };

            if let Some(frame) = self.edit_keyframe_for_element(assigned_element_id).await {
                // We need to know the properties of all of the elements in the current frame (we need to work backwards to generate the grouped element)
                let when                        = frame.future_sync(|frame| async move { frame.start }.boxed()).await.unwrap();
                let elements_with_properties    = Self::frame_elements_with_properties(Arc::clone(&frame), when).await;

                // Nothing to do if there are no properties
                if elements_with_properties.len() == 0 {
                    return ReversedEdits::empty();
                }

                let (updates, reversed) = frame.future_sync(move |frame| {
                    async move {
                        let mut reversed    = ReversedEdits::new();
                        let layer_id        = frame.layer_id;

                        // Find the brush properties for the selected element. These are usually at the end, so a linear search like this should be fine
                        let source_element_properties = elements_with_properties.iter().rev()
                            .filter(|elem| elem.0.element.id() == source_element_id)
                            .map(|elem| elem.1.clone())
                            .nth(0)
                            .unwrap_or_else(|| Arc::new(VectorProperties::default()));
                        let source_brush = &source_element_properties.brush;

                        // Fetch the element from the frame
                        let source_wrapper  = frame.elements.get(&source_element_id).cloned();
                        let mut updates     = PendingStorageChange::new();

                        let source_wrapper  = match source_wrapper {
                            Some(wrapper)   => wrapper,
                            None            => { return (updates, reversed); }
                        };

                        // Collide other elements in the frame with this element
                        // Only brush stroke elements can be combined at the moment
                        match &source_wrapper.element {
                            Vector::BrushStroke(_) => {
                                // Attempt to combine the element we fetched with the rest of the frame
                                let mut combined_element            = None;
                                for (combine_with_wrapper, properties) in elements_with_properties.iter().rev() {
                                    use self::CombineResult::*;

                                    // Ignore the element we're merging
                                    // TODO: consider ignoring the elements above the element we're merging too
                                    if combine_with_wrapper.element.id() == source_element_id {
                                        continue;
                                    }

                                    // The 'combined so far' vector is either just our brush stroke, or what we've got from the combination we've built up so far
                                    let combined_so_far = combined_element.as_ref().unwrap_or_else(|| &source_wrapper.element);

                                    let new_combined = match source_brush.combine_with(combined_so_far, &source_element_properties, &combine_with_wrapper.element, &*properties) {
                                        NewElement(new_combined)    => {
                                            // Unlink the element from the frame (brushes typically put their new element into a group so
                                            // this will set up the element in a way that's appropriate for that)
                                            reversed.extend(ReversedEdits::with_relinked_element(layer_id, &combine_with_wrapper, &|id| frame.elements.get(&id).cloned()));
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
                                let replacement_element = frame.elements.get(&source_element_id).cloned();
                                if let (Some(mut combined_element), Some(mut replacement_element)) = (combined_element, replacement_element) {
                                    // Reversing deletes and recreates the element (after all the internal elements have been moved back to where they belong)
                                    reversed.add_to_start(ReversedEdits::with_recreated_wrapper(layer_id, &replacement_element, &|id| frame.elements.get(&id).cloned()));
                                    reversed.insert(0, AnimationEdit::Element(vec![source_element_id], ElementEdit::Delete));

                                    // The new combined element inherits the ID of the original source element
                                    combined_element.set_id(source_element_id);

                                    if combined_element.sub_element_ids().into_iter().any(|id| id == source_element_id) {
                                        // Create a new set of sub-elements where the source ID is unassigned
                                        let new_elements = combined_element.sub_elements().map(|elem| {
                                            if elem.id() == source_element_id {
                                                elem.with_id(ElementId::Unassigned)
                                            } else {
                                                elem.clone()
                                            }
                                        }).collect();

                                        // Re-group the elements
                                        combined_element = match combined_element {
                                            Vector::Group(group_elem) => {
                                                let mut new_group = GroupElement::new(source_element_id, group_elem.group_type(), Arc::new(new_elements));
                                                if let Some(hint) = group_elem.hint_path() { new_group.set_hint_path(hint); }

                                                Vector::Group(new_group)
                                            }

                                            other => other
                                        }
                                    }

                                    // Replace the source element with the combined element
                                    replacement_element.element = combined_element;

                                    // Update it in the storage
                                    frame.invalidate();
                                    updates.push_element(assigned_element_id, replacement_element.clone());
                                    frame.elements.insert(source_element_id, replacement_element);

                                    // Make sure the parents are set correctly
                                    updates.extend(frame.update_parents(source_element_id));
                                } else {
                                    // If nothing was generated then any updates that might have been generated are not valid
                                    updates = PendingStorageChange::new();
                                }
                            }

                            _ => { }
                        }

                        // The result is the list of updates we want to perform
                        (updates, reversed)
                    }.boxed()
                }).await.unwrap();

                // Assign IDs to the elements in the updates
                let mut updates     = updates;
                let mut reversed    = reversed;
                let mut new_updates = PendingStorageChange::new();

                for update_wrapper in updates.pending_element_wrappers() {
                    // If there are any elements without IDs in the pending changes, add them
                    let assign_id_changes = self.assign_ids_to_elements(update_wrapper, &mut reversed).await;

                    if !assign_id_changes.is_empty() {
                        // Append the changes created for the element
                        new_updates.extend(assign_id_changes);
                    }
                }

                if !new_updates.is_empty() {
                    updates.extend(new_updates);
                }
                
                // Send the updates to storage
                self.request(updates).await;

                reversed
            } else {
                // No frame
                ReversedEdits::empty()
            }
        }
    }
}
