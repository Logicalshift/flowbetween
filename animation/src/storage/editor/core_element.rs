use super::keyframe_core::*;
use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::super::storage_api::*;
use super::super::super::traits::*;
use super::super::super::serializer::*;

use futures::prelude::*;
use ::desync::*;

use std::sync::*;
use std::collections::{HashSet};

///
/// Possible updates that can be made by the 
///
pub (super) enum ElementUpdate {
    /// Update the element wrapper
    ChangeWrapper(ElementWrapper),

    /// Add the specified attachments and attach them to the keyframe if they're not already present
    AddAttachments(Vec<ElementId>),

    /// Remove the specified attachments (attachments are left on the keyframe)
    RemoveAttachments(Vec<ElementId>),

    /// Perform other updates, according to the specified storage command
    Other(Vec<StorageCommand>)
}

impl StreamAnimationCore {
    ///
    /// Performs an element edit on this animation
    ///
    pub fn element_edit<'a>(&'a mut self, element_ids: &'a Vec<ElementId>, element_edit: &'a ElementEdit) -> impl 'a+Send+Future<Output=()> {
        async move {
            use self::ElementEdit::*;
            use self::ElementUpdate::*;

            let element_ids = element_ids.iter().map(|elem| elem.id()).flatten().collect();

            match element_edit {
                AddAttachment(attach_id)        => { self.update_elements(element_ids, |wrapper| { AddAttachments(vec![*attach_id]) }).await; }
                RemoveAttachment(attach_id)     => { self.update_elements(element_ids, |wrapper| { RemoveAttachments(vec![*attach_id]) }).await; }
                SetControlPoints(new_points)    => { self.update_elements(element_ids, |mut wrapper| { wrapper.element = wrapper.element.with_adjusted_control_points(new_points.clone()); ChangeWrapper(wrapper) }).await; }
                SetPath(new_path)               => { self.update_elements(element_ids, |mut wrapper| { wrapper.element = wrapper.element.with_path_components(new_path.iter().cloned()); ChangeWrapper(wrapper) }).await; }
                Order(ordering)                 => { self.order_elements(element_ids, *ordering).await; }
                Delete                          => { self.request(element_ids.into_iter().map(|id| StorageCommand::DeleteElement(id)).collect()).await; }
                DetachFromFrame                 => { self.request(element_ids.into_iter().map(|id| StorageCommand::DetachElementFromLayer(id)).collect()).await; }
            }
        }
    }

    ///
    /// Performs an update on an element in a keyframe
    ///
    fn perform_update<'a>(&'a mut self, element_id: i64, update: ElementUpdate, keyframe: Option<Arc<Desync<KeyFrameCore>>>) -> impl 'a+Send+Future<Output=Vec<StorageCommand>>+Send {
        async move {
            let mut updates = vec![];

            match update {
                ElementUpdate::ChangeWrapper(updated_element) => {
                    // Generate the update of the serialized element
                    let mut serialized  = String::new();
                    updated_element.serialize(&mut serialized);

                    updates.push(StorageCommand::WriteElement(element_id, serialized));

                    // Replace the element in the keyframe
                    keyframe.map(|keyframe| {
                        keyframe.desync(move |keyframe| {
                            keyframe.elements.lock().unwrap()
                                .insert(ElementId::Assigned(element_id), updated_element);
                        });
                    });
                }

                ElementUpdate::AddAttachments(attachments) => {
                    // Update the attachments in the keyframe (elements outside of keyframes must not have attachments)
                    keyframe.map(|keyframe| keyframe.sync(|keyframe| {
                        // Fetch the keyframe elements
                        let mut elements = keyframe.elements.lock().unwrap();

                        // Add the attachments to the keyframe
                        let attachment_ids = attachments.iter()
                            .filter(|attachment_id| !elements.contains_key(attachment_id))
                            .filter_map(|attachment_id| attachment_id.id());

                        updates.extend(attachment_ids.map(|attachment_id| StorageCommand::AttachElementToLayer(keyframe.layer_id, attachment_id, keyframe.start)));

                        // Add an attachment to the element
                        elements.get_mut(&ElementId::Assigned(element_id))
                            .map(|element_wrapper| {
                                // Add the attachment
                                element_wrapper.attachments.extend(attachments.clone());

                                // Generate the update of the serialized element
                                let mut serialized  = String::new();
                                element_wrapper.serialize(&mut serialized);

                                updates.push(StorageCommand::WriteElement(element_id, serialized));
                            });
                    }));
                }

                ElementUpdate::RemoveAttachments(attachments) => {
                    // Hash the attachments
                    let attachments = attachments.into_iter().collect::<HashSet<_>>();

                    // Update the attachments in the keyframe (elements outside of keyframes must not have attachments)
                    keyframe.map(|keyframe| keyframe.sync(|keyframe| {
                        // Fetch the keyframe elements
                        let mut elements = keyframe.elements.lock().unwrap();

                        // Remove the attachments from the element
                        elements.get_mut(&ElementId::Assigned(element_id))
                            .map(|element_wrapper| {
                                // Add the attachment
                                element_wrapper.attachments.retain(|attachment_id| !attachments.contains(attachment_id));

                                // Generate the update of the serialized element
                                let mut serialized  = String::new();
                                element_wrapper.serialize(&mut serialized);

                                updates.push(StorageCommand::WriteElement(element_id, serialized));
                            });
                    }));
                }

                ElementUpdate::Other(cmds) => {
                    updates = cmds;
                }
            }

            updates
        }
    }

    ///
    /// Updates a one or more elements via an update function
    ///
    pub fn update_elements<'a, UpdateFn>(&'a mut self, element_ids: Vec<i64>, update_fn: UpdateFn) -> impl 'a+Future<Output=()>
    where UpdateFn: 'a+Send+Sync+Fn(ElementWrapper) -> ElementUpdate {
        async move {
            // Update the elements that are returned
            let mut updates = vec![];

            // Build a hashset of the remaining elements
            let mut remaining = element_ids.iter().cloned().collect::<HashSet<_>>();

            // ... until we've removed all the remaining elements...
            while let Some(root_element) = remaining.iter().nth(0).cloned() {
                // Fetch the keyframe that the root element is in
                if let Some(keyframe) = self.edit_keyframe_for_element(root_element).await {
                    // ... the element is in a keyframe, and we retrieved that keyframe
                    let to_process = remaining.iter().cloned().collect::<Vec<_>>();

                    for element_id in to_process {
                        // Try to retrieve the element from the keyframe
                        let existing_element = keyframe.future(move |keyframe| {
                            async move {
                                let elements = keyframe.elements.lock().unwrap();
                                elements.get(&ElementId::Assigned(element_id)).cloned()
                            }.boxed()
                        }).await;

                        // Update the existing element if we managed to retrieve it
                        if let Ok(Some(existing_element)) = existing_element {
                            // Process via the update function
                            let updated_element = update_fn(existing_element);
                            let element_updates = self.perform_update(element_id, updated_element, Some(keyframe.clone())).await;
                            updates.extend(element_updates);

                            // Remove the element from the remaining list so we don't try to update it again
                            remaining.remove(&element_id);
                        }
                    }
                } else {
                    // The element is independent of a keyframe. These elements cannot be edited if they depend on others (at the moment)
                    if let Some(StorageResponse::Element(_, element)) = self.request_one(StorageCommand::ReadElement(root_element)).await {
                        // Decode the element (without looking up any dependencies)
                        let element = ElementWrapper::deserialize(ElementId::Assigned(root_element), &mut element.chars())
                            .and_then(|element| element.resolve(&mut |_| None));

                        if let Some(element) = element {
                            // Update the element
                            let updated_element = update_fn(element);
                            updates.extend(self.perform_update(root_element, updated_element, None).await);
                        }
                    }
                }

                // The root element is always removed from the remaining list even if we couldn't get its keyframe
                remaining.remove(&root_element);
            }

            // Update all of the elements
            self.request(updates).await;
        }
    }

    ///
    /// Re-orders a set of elements in their keyframes
    ///
    pub fn order_elements<'a>(&'a mut self, element_ids: Vec<i64>, ordering: ElementOrdering) -> impl 'a + Future<Output=()>  {
        async move {
            // List of updates to perform as a result of this ordering operation
            let mut updates = vec![];

            // Order each element in turn
            for element_id in element_ids {
                // Load the keyframe for this element if none is currently loaded
                let current_keyframe = self.edit_keyframe_for_element(element_id).await;

                // Order the element in the keyframe
                let maybe_updates = current_keyframe.and_then(|keyframe| 
                    keyframe.sync(|keyframe| 
                        keyframe.order_element(ElementId::Assigned(element_id), ordering)));

                // Add the updates
                maybe_updates.map(|keyframe_updates| updates.extend(keyframe_updates));
            }

            // Perform the updates
            self.request(updates).await;
        }
    }
}
