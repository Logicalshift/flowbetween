use super::keyframe_core::*;
use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::pending_storage_change::*;
use crate::storage::storage_api::*;
use crate::undo::*;
use crate::traits::*;
use crate::serializer::*;

use futures::prelude::*;
use ::desync::*;

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashSet};

///
/// Possible updates that can be made to elements in storage
///
pub (super) enum ElementUpdate {
    /// Update the element wrapper
    ChangeWrapper(ElementWrapper),

    /// Add the specified attachments and attach them to the keyframe if they're not already present
    AddAttachments(Vec<ElementId>),

    /// Remove the specified attachments (attachments are left on the keyframe)
    RemoveAttachments(Vec<ElementId>),

    /// Unlinks the specified element from its frame
    Unlink,

    /// Perform other updates, according to the specified storage command
    Other(Vec<StorageCommand>)
}

impl StreamAnimationCore {
    ///
    /// Performs an element edit on this animation
    ///
    pub fn element_edit<'a>(&'a mut self, element_ids: &'a Vec<ElementId>, element_edit: &'a ElementEdit) -> impl 'a+Send+Future<Output=ReversedEdits> {
        async move {
            use self::ElementEdit::*;
            use self::ElementUpdate::*;

            let wrapped_element_ids = element_ids;
            let element_ids         = element_ids.iter().map(|elem| elem.id()).flatten().collect::<Vec<_>>();

            match element_edit {
                AttachTo(element_id)                => {
                    if let ElementId::Assigned(element_id) = element_id {
                        self.update_elements(vec![*element_id], |_wrapper| { AddAttachments(wrapped_element_ids.clone()) }).await;
                        ReversedEdits::with_edits(
                            wrapped_element_ids
                                .iter()
                                .cloned()
                                .map(|attachment_id| {
                                    AnimationEdit::Element(vec![ElementId::Assigned(*element_id)], ElementEdit::RemoveAttachment(attachment_id))
                                })
                        )
                    } else {
                        ReversedEdits::empty()
                    }
                }

                AddAttachment(attach_id)            => { 
                    self.update_elements(element_ids, |_wrapper| { AddAttachments(vec![*attach_id]) }).await;
                    ReversedEdits::with_edit(AnimationEdit::Element(wrapped_element_ids.clone(), ElementEdit::RemoveAttachment(*attach_id)))
                }

                RemoveAttachment(attach_id)         => { 
                    self.update_elements(element_ids, |_wrapper| { RemoveAttachments(vec![*attach_id]) }).await; 
                    ReversedEdits::with_edit(AnimationEdit::Element(wrapped_element_ids.clone(), ElementEdit::AddAttachment(*attach_id)))
                }

                SetPath(new_path)                   => { 
                    let mut reversed_edits = ReversedEdits::new();

                    self.update_elements(element_ids, |mut wrapper| { 
                        let id          = wrapper.element.id(); 
                        reversed_edits.push(AnimationEdit::Element(vec![id], ElementEdit::SetPath(wrapper.element.path_components())));

                        wrapper.element = wrapper.element.with_path_components(new_path.iter().cloned()); ChangeWrapper(wrapper) 
                    }).await; 

                    reversed_edits
                }

                Order(ordering)                     => { 
                    self.order_elements(element_ids, *ordering).await
                }

                Group(group_id, group_type)         => { 
                    self.group_elements(element_ids, *group_id, *group_type).await; 
                    ReversedEdits::with_edit(AnimationEdit::Element(vec![*group_id], ElementEdit::Ungroup))
                }
                
                Ungroup                             => { 
                    let mut reversed = ReversedEdits::new();

                    for id in element_ids {
                        reversed.add_to_start(self.ungroup_element(ElementId::Assigned(id)).await); 
                    }

                    reversed
                }

                ConvertToPath                       => {
                    let mut reversed = ReversedEdits::new();

                    for id in element_ids {
                        reversed.add_to_start(self.convert_element_to_path(ElementId::Assigned(id)).await);
                    }

                    reversed
                }

                CollideWithExistingElements         => { 
                    let mut reversed = ReversedEdits::new();

                    for id in element_ids.iter() {
                        reversed.add_to_start(self.collide_with_existing_elements(ElementId::Assigned(*id)).await);
                    }

                    reversed
                }

                Delete                              => {
                    // Create the undo operation for each of the deleted elements
                    let mut reversed        = ReversedEdits::new();
                    let mut element_frames  = vec![];

                    for element_id in element_ids.iter().cloned() {
                        if let Some(frame) = self.edit_keyframe_for_element(element_id).await {
                            // Request the element from the frame
                            let frame_reverse = frame.future_sync(move |frame| {
                                async move {
                                    let wrapper = frame.elements.get(&ElementId::Assigned(element_id))?;
                                    Some(ReversedEdits::with_recreated_wrapper(frame.layer_id, wrapper, &|id| frame.elements.get(&id).cloned()))
                                }.boxed()
                            }).await.unwrap();

                            frame_reverse.map(|frame_reverse| reversed.add_to_start(frame_reverse));

                            // Remember the frames for later
                            element_frames.push((element_id, frame));
                        }
                    }

                    // If the element is attached to another element, remove it from the attachment list
                    self.remove_from_attachments(&element_ids).await;

                    // Delete from storage
                    self.request(element_ids.iter().cloned().map(|id| StorageCommand::DeleteElement(id))).await;

                    // Remove the element from the edit frames, so it's not cached
                    element_frames.into_iter()
                        .for_each(|(element_id, frame)| {
                            frame.desync(move |frame| { frame.elements.remove(&ElementId::Assigned(element_id)); });
                        });

                    reversed
                }

                DetachFromFrame                     => {
                    let mut reversed = ReversedEdits::new();

                    // Re-link all of the elements when undoing this action
                    for element_id in element_ids.iter().cloned() {
                        if let Some(frame) = self.edit_keyframe_for_element(element_id).await {
                            // Request the element from the frame
                            let frame_reverse = frame.future_sync(move |frame| {
                                async move {
                                    let wrapper = frame.elements.get(&ElementId::Assigned(element_id))?;
                                    Some(ReversedEdits::with_relinked_element(frame.layer_id, wrapper, &|id| frame.elements.get(&id).cloned()))
                                }.boxed()
                            }).await.unwrap();

                            frame_reverse.map(|frame_reverse| reversed.add_to_start(frame_reverse));
                        }
                    }

                    // If the element is attached to another element, remove it from the attachment list
                    reversed.extend(self.remove_from_attachments(&element_ids).await);

                    // Remove from the list of elements attached to a particular layer
                    self.request(element_ids.into_iter().map(|id| StorageCommand::DetachElementFromLayer(id))).await; 

                    reversed
                },

                Transform(transformations)          => {
                    self.transform_elements(&element_ids, transformations).await
                }

                SetControlPoints(new_points, when)  => {
                    self.set_control_points(&element_ids, new_points, *when).await
                }

                SetAnimationDescription(new_description) => {
                    let mut reversed = ReversedEdits::new();

                    self.update_elements(element_ids, |mut wrapper| { 
                        if let Vector::AnimationRegion(animation_element) = &wrapper.element {
                            let id          = wrapper.element.id(); 

                            // Reversal is to set the description back to what it was before
                            let description = animation_element.description().clone();
                            reversed.insert(0, AnimationEdit::Element(vec![id], ElementEdit::SetAnimationDescription(description)));

                            wrapper.element = Vector::AnimationRegion(AnimationElement::new(id, new_description.clone())); 
                            ChangeWrapper(wrapper) 
                        } else {
                            // Not an animation element: do nothing
                            ChangeWrapper(wrapper)
                        }
                    }).await;

                    reversed
                }

                SetAnimationBaseType(new_base_type) => {
                    let mut reversed = ReversedEdits::new();

                    self.update_elements(element_ids, |mut wrapper| {
                        match &mut wrapper.element {
                            Vector::AnimationRegion(region) => {
                                let old_type            = region.effect().base_animation_type();
                                reversed.insert(0, AnimationEdit::Element(vec![region.id()], ElementEdit::SetAnimationBaseType(old_type)));

                                let updated_effect      = region.effect().update_effect_animation_type(*new_base_type);
                                *region.effect_mut()    = updated_effect;
                            }

                            _ => { }
                        }

                        ChangeWrapper(wrapper)
                    }).await;

                    reversed
                }

                AddAnimationEffect(new_effect_type) => { 
                    let mut reversed = ReversedEdits::new();

                    self.update_elements(element_ids, |mut wrapper| {
                        match &mut wrapper.element {
                            Vector::AnimationRegion(region) => {
                                // Reversal is to set the description back to what it was before
                                let id          = region.id();
                                let description = region.description().clone();
                                reversed.insert(0, AnimationEdit::Element(vec![id], ElementEdit::SetAnimationDescription(description)));

                                // Add the effect and update the region
                                let updated_effect      = region.effect().add_new_effect(*new_effect_type);
                                *region.effect_mut()    = updated_effect;
                            }

                            _ => { }
                        }

                        ChangeWrapper(wrapper)
                    }).await;

                    reversed
                }

                ReplaceAnimationEffect(address, description) => {
                    let mut reversed = ReversedEdits::new();

                    self.update_elements(element_ids, |mut wrapper| {
                        match &mut wrapper.element {
                            Vector::AnimationRegion(region) => {
                                let id          = region.id();
                                let subeffect   = region.effect().sub_effects().into_iter().filter(|item| &item.address() == address).nth(0);
                                if let Some(subeffect) = subeffect {
                                    // Reversal replaces the effect with the original effect
                                    reversed.insert(0, AnimationEdit::Element(vec![id], ElementEdit::ReplaceAnimationEffect(address.clone(), subeffect.effect_description().clone())));

                                    // Update the effect
                                    let updated_effect      = region.effect().replace_sub_effect(&subeffect, description.clone());
                                    *region.effect_mut()    = updated_effect;
                                }
                            }

                            _ => { }
                        }

                        ChangeWrapper(wrapper)
                    }).await;

                    reversed
                }
            }
        }
    }

    ///
    /// When deleting or detaching an element, we might find that it has attachments or is attached to other elements.
    /// This will remove the element from the attachment lists of those related elements.
    /// 
    /// Note that this might leave elements that are no longer attached to anything: this presently does not clean
    /// up these elements. The reversed edits will add the element back as an attachment but won't re-link it (use
    /// `ReversedEdits::with_relinked_element` to do that if needed)
    ///
    pub fn remove_from_attachments<'a>(&'a mut self, element_ids: &'a Vec<i64>) -> impl 'a+Send+Future<Output=ReversedEdits> {
        async move {
            let mut attachments         = vec![];
            let mut attached_to         = vec![];

            // Use update_elements to read the attachments/attached_to values for the elements that are being deleted
            let mut reversed = ReversedEdits::new();

            self.update_elements(element_ids.clone(), |wrapper| {
                reversed.push(AnimationEdit::Element(wrapper.attachments.clone(), ElementEdit::AttachTo(wrapper.element.id())));
                reversed.push(AnimationEdit::Element(wrapper.attached_to.clone(), ElementEdit::AddAttachment(wrapper.element.id())));

                attachments.extend(wrapper.attachments.into_iter().map(|id| id.id()).flatten());
                attached_to.extend(wrapper.attached_to.into_iter().map(|id| id.id()).flatten());

                ElementUpdate::Unlink
            }).await;

            // Remove the element from anything it's attached to or is attached to it
            if attachments.len() > 0 || attached_to.len() > 0 {
                // Hash set of the elements that are being deleted
                let attachments_to_remove   = element_ids.iter().map(|id| ElementId::Assigned(*id)).collect::<HashSet<_>>();

                // Remove the deleted elements from anything they're attached to
                self.update_elements(attachments, |wrapper| Self::remove_attachments(wrapper, &attachments_to_remove)).await;
                self.update_elements(attached_to, |wrapper| Self::remove_attachments(wrapper, &attachments_to_remove)).await;
            }

            reversed
        }
    }

    ///
    /// Given an attachment ID, removes it from the attachments (and attached_to) items for an element
    ///
    fn remove_attachments(wrapper: ElementWrapper, attachments_to_remove: &HashSet<ElementId>) -> ElementUpdate {
        let mut wrapper     = wrapper;

        // Count the attachments so we know if we've removed any
        let num_attachments = wrapper.attachments.len();
        let num_attached_to = wrapper.attached_to.len();

        // Remove any instances of the specified attachments
        wrapper.attachments.retain(|id| !attachments_to_remove.contains(id));
        wrapper.attached_to.retain(|id| !attachments_to_remove.contains(id));

        if num_attachments != wrapper.attachments.len() || num_attached_to != wrapper.attached_to.len() {
            // Update the wrapper
            ElementUpdate::ChangeWrapper(wrapper)
        } else {
            // Do nothing as the attachment wasn't in the list
            ElementUpdate::Other(vec![])
        }
    }

    ///
    /// Reads a set of elements (invalid elements or elements with references not in the keyframe will not be returned)
    ///
    fn read_elements<'a>(&'a mut self, element_ids: &'a Vec<i64>, keyframe: Option<Arc<Desync<KeyFrameCore>>>) -> impl 'a+Send+Future<Output=Vec<ElementWrapper>> {
        async move {
            // No elements are returned if no IDs are passed in
            if element_ids.len() == 0 {
                return vec![];
            }

            // Request the serialized elements from storage
            let response = self.request(element_ids.iter().map(|id| StorageCommand::ReadElement(*id)).collect::<Vec<_>>()).await;

            // Deserialize each element in the response (assume they don't refer to each other)
            let mut elements = vec![];

            for elem_response in response.unwrap_or_else(|| vec![]) {
                match elem_response {
                    StorageResponse::Element(id, serialized) => { 
                        // Deserialize this element
                        let resolver    = ElementWrapper::deserialize(ElementId::Assigned(id), &mut serialized.chars());
                        let element     = if let Some(keyframe) = keyframe.as_ref() {
                            // Resolve with the existing elements in the keyframe
                            resolver.and_then(|resolver| resolver.resolve(&mut |id| {
                                keyframe.sync(move |keyframe| keyframe.elements.get(&id)
                                    .map(|wrapper| wrapper.element.clone()))
                            }))
                        } else {
                            // Resolve with no extra elements
                            resolver.and_then(|resolver| resolver.resolve(&mut |_| None))
                        };

                        // Add the deserialized element to the results list
                        element.map(|element| elements.push(element));
                    },

                    _   => { 
                        // Other responses are ignored
                    }
                }
            }

            elements
        }
    }

    ///
    /// Performs an update on an element in a keyframe
    ///
    fn perform_element_update<'a>(&'a mut self, element_id: i64, update: ElementUpdate, keyframe: Option<Arc<Desync<KeyFrameCore>>>) -> impl 'a+Send+Future<Output=PendingStorageChange>+Send {
        async move {
            let mut updates = PendingStorageChange::new();

            match update {
                ElementUpdate::ChangeWrapper(updated_element) => {
                    // Generate the update of the serialized element
                    updates.push_element(element_id, updated_element.clone());

                    // Replace the element in the keyframe
                    keyframe.map(|keyframe| {
                        keyframe.desync(move |keyframe| {
                            keyframe.elements
                                .insert(ElementId::Assigned(element_id), updated_element);
                        });
                    });
                }

                ElementUpdate::AddAttachments(attachments) => {
                    // Update the attachments in the keyframe (elements outside of keyframes must not have attachments)
                    if let Some(keyframe) = keyframe {
                        // Get the missing elements
                        let missing_attachment_ids = keyframe.sync(|keyframe| {
                            // Add the attachments to the keyframe (this finds the attachment IDs tha are not already in the keyframe)
                            let attachment_ids = attachments.iter()
                                .filter(|attachment_id| !keyframe.elements.contains_key(attachment_id))
                                .filter_map(|attachment_id| attachment_id.id())
                                .collect::<Vec<_>>();

                            attachment_ids
                        });

                        // Read the attachments that are missing from the keyframe
                        let missing_attachments = self.read_elements(&missing_attachment_ids, Some(keyframe.clone())).await;

                        keyframe.sync(|keyframe| {
                            // Add the missing attachments to the keyframe
                            for attachment in missing_attachments.iter() {
                                let id = attachment.element.id();
                                keyframe.elements.insert(id, attachment.clone());
                            }
                            keyframe.invalidate();

                            // Attach the elements to the layer
                            updates.extend(missing_attachment_ids.iter().map(|attachment_id| StorageCommand::AttachElementToLayer(keyframe.layer_id, *attachment_id, keyframe.start)));

                            // Attach the elements
                            updates.extend(keyframe.add_attachment(ElementId::Assigned(element_id), &attachments));
                        });
                    }
                }

                ElementUpdate::RemoveAttachments(attachments) => {
                    // Hash the attachments
                    let attachments = attachments.into_iter().collect::<HashSet<_>>();

                    // Update the attachments in the keyframe (elements outside of keyframes must not have attachments)
                    keyframe.map(|keyframe| keyframe.sync(|keyframe| {
                        // Removes the element from the attachments
                        attachments.iter()
                            .for_each(|attachment_id| {
                                // Remove the element
                                keyframe.elements.get_mut(&attachment_id)
                                    .map(|attachment_wrapper| {
                                        // Remove the element from the wrapper
                                        attachment_wrapper.attached_to.retain(|existing_id| existing_id != &ElementId::Assigned(element_id));

                                        // Send back to the storage with the attachment removed
                                        // TODO: if there are no attachments left, consider removing the element from the keyframe
                                        // (presently doesn't work as brush properties don't have their references reversed this way)
                                        attachment_id.id().map(|attachment_id| {
                                            updates.push_element(attachment_id, attachment_wrapper.clone());
                                        });
                                    });
                            });

                        // Remove the attachments from the element
                        keyframe.elements.get_mut(&ElementId::Assigned(element_id))
                            .map(|element_wrapper| {
                                // Add the attachment
                                element_wrapper.attachments.retain(|attachment_id| !attachments.contains(attachment_id));

                                // Generate the update of the serialized element
                                updates.push_element(element_id, element_wrapper.clone());
                            });

                        keyframe.invalidate();
                    }));
                }

                ElementUpdate::Unlink => {
                    if let Some(keyframe) = keyframe {
                        // Generate the unlink updates on the keyframe
                        let unlink_updates = keyframe.future_sync(move |frame| {
                            async move {
                                frame.unlink_element(ElementId::Assigned(element_id))
                            }.boxed()
                        }).await.unwrap();

                        // Add to the updates
                        updates.extend(unlink_updates)
                    }
                }

                ElementUpdate::Other(cmds) => {
                    updates.extend(cmds);
                }
            }

            updates
        }
    }

    ///
    /// Updates a one or more elements via an update function
    ///
    pub fn update_elements<'a, UpdateFn>(&'a mut self, element_ids: Vec<i64>, mut update_fn: UpdateFn) -> impl 'a+Future<Output=()>
    where UpdateFn: 'a+Send+Sync+FnMut(ElementWrapper) -> ElementUpdate {
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
                        let existing_element = keyframe.future_sync(move |keyframe| {
                            async move {
                                keyframe.elements.get(&ElementId::Assigned(element_id)).cloned()
                            }.boxed()
                        }).await;

                        // Update the existing element if we managed to retrieve it
                        if let Ok(Some(existing_element)) = existing_element {
                            // Process via the update function
                            let updated_element = update_fn(existing_element);
                            let element_updates = self.perform_element_update(element_id, updated_element, Some(keyframe.clone())).await;
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
                            updates.extend(self.perform_element_update(root_element, updated_element, None).await);
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
    /// Updates the control points for a list of elements
    ///
    pub fn set_control_points<'a>(&'a mut self, element_ids: &'a Vec<i64>, new_control_points: &'a Vec<(f32, f32)>, when: Duration) -> impl 'a + Future<Output=ReversedEdits> {
        async move {
            let mut reversed    = ReversedEdits::new();
            let mut updates     = vec![];

            // Need to process the elements individually with their properties
            for element_id in element_ids.iter().cloned() {
                if let Some(frame) = self.edit_keyframe_for_element(element_id).await {
                    let new_points  = new_control_points.clone();

                    // Use the frame to update the element
                    let maybe_updates = frame.future_sync(move |frame| {
                        async move {
                            // Fetch the element wrapper
                            let mut wrapper     = frame.elements.get(&ElementId::Assigned(element_id))?.clone();

                            // Work out the properties for this element
                            let properties      = frame.apply_properties_for_element(&wrapper.element, Arc::new(VectorProperties::default()), when);
                            
                            // Reversal just restores the original control points for this element
                            let control_points  = wrapper.element.control_points(&*properties);
                            let control_points  = control_points.into_iter().map(|cp| cp.position()).map(|(x, y)| (x as f32, y as f32));
                            let reversed        = ReversedEdits::with_edit(AnimationEdit::Element(vec![ElementId::Assigned(element_id)], ElementEdit::SetControlPoints(control_points.collect(), when)));

                            // Update the control points
                            wrapper.element     = wrapper.element.with_adjusted_control_points(new_points, &*properties);

                            // Generate the updates for this element
                            let updates         = vec![StorageCommand::WriteElement(element_id, wrapper.serialize_to_string())]; 
                            frame.elements.insert(ElementId::Assigned(element_id), wrapper);
                            frame.invalidate();

                            Some((updates, reversed))
                        }.boxed()
                    }).await.unwrap();

                    // Add the updates to the overall list
                    if let Some((element_updates, element_reversed)) = maybe_updates {
                        reversed.add_to_start(element_reversed);
                        updates.extend(element_updates);
                    }
                }
            }

            // Send the updates to storage
            self.request(updates).await;

            reversed
        }
    }

    ///
    /// Moves a set of elements into a single group
    ///
    pub fn group_elements<'a>(&'a mut self, element_ids: Vec<i64>, group_id: ElementId, group_type: GroupType) -> impl 'a + Future<Output=()> {
        async move {
            // Nothing to do if there are no elements to group
            if element_ids.len() == 0 {
                return;
            }

            // Fetch the frame for the first element
            let frame = self.edit_keyframe_for_element(element_ids[0]).await;
            let frame = match frame {
                Some(frame) => frame,
                None        => { return; }
            };

            // Assign an ID to the group if none is supplied
            let mut group_id = group_id;
            if group_id.is_unassigned() {
                group_id = self.assign_element_id(group_id).await;
            }

            let group_id = match group_id.id() {
                Some(id)    => id,
                None        => { return; }
            };

            let updates = frame.future_sync(move |frame| {
                async move {
                    let mut updates         = PendingStorageChange::new();
                    let mut group_elements  = vec![];

                    let first_element   = frame.elements.get(&ElementId::Assigned(element_ids[0])).unwrap().clone();
                    let mut start_time  = first_element.start_time;
                    let mut order_after = first_element.order_after;
                    let parent          = first_element.parent;

                    // Find all the elements and unlink them
                    for element_id in element_ids.iter() {
                        if let Some(element) = frame.elements.get_mut(&ElementId::Assigned(*element_id)) {
                            // If this is the current order_after element, move it behind this element
                            // TODO: this is simple but buggy with some group orderings (we can miss that this used an element that )
                            if order_after == Some(ElementId::Assigned(*element_id)) {
                                order_after = element.order_after;
                            }

                            // The start time of the group is the minimum of all elements
                            start_time = Duration::min(start_time, element.start_time);

                            // Add to the elements that go in our final group
                            group_elements.push(element.clone());

                            // Unlink the element
                            let unlink = frame.unlink_element(ElementId::Assigned(*element_id));
                            updates.extend(unlink);

                            // Set the parent of the element to be our new group element
                            let element = frame.elements.get_mut(&ElementId::Assigned(*element_id)).unwrap();

                            if element.parent != Some(ElementId::Assigned(group_id)) {
                                element.parent = Some(ElementId::Assigned(group_id));
                                updates.push_element(*element_id, element.clone());
                            }
                        }
                    }

                    // Create the group element: properties are from the first element
                    let group       = group_elements.iter().map(|wrapper| wrapper.element.clone()).collect();
                    let group       = GroupElement::new(ElementId::Assigned(group_id), group_type, Arc::new(group));
                    let group       = Vector::Group(group);
                    let mut group   = ElementWrapper::attached_with_element(group, start_time);

                    // Normal groups take their properties from their internal elements. Other groups use
                    // the properties of their first element.
                    if group_type != GroupType::Normal {
                        for attachment_id in first_element.attachments.iter() {
                            // The group should have the same attachments as its first element
                            updates.extend(frame.add_attachment(ElementId::Assigned(group_id), &vec![*attachment_id]));
                            group.attachments.push(*attachment_id);
                        }
                    }

                    // Add the new group to the updates
                    updates.push_element(group_id, group.clone());
                    updates.push(StorageCommand::AttachElementToLayer(frame.layer_id, group_id, start_time));

                    // Add the group to the frame
                    frame.invalidate();
                    frame.elements.insert(ElementId::Assigned(group_id), group);

                    // Insert the group into the frame in place of the original element
                    updates.extend(frame.order_after(ElementId::Assigned(group_id), parent, order_after));

                    updates
                }.boxed()
            }).await.unwrap();

            // Send the updates to storage
            self.request(updates).await;
        }
    }

    ///
    /// Given an element that represents a group, ungroups it and moves all the elements in the group into the parent
    /// element
    ///
    pub fn ungroup_element<'a>(&'a mut self, group_element_id: ElementId) -> impl 'a+Future<Output=ReversedEdits> {
        async move {
            let group_element_id = match group_element_id.id() {
                Some(id)    => id,
                None        => { return ReversedEdits::empty(); }
            };

            // Fetch the frame for the grouped element
            let frame = self.edit_keyframe_for_element(group_element_id).await;
            let frame = match frame {
                Some(frame) => frame,
                None        => { return ReversedEdits::empty(); }
            };

            // Modify the frame
            let (reversed, updates) = frame.future_sync(move |frame| {
                let mut reversed = ReversedEdits::new();

                async move {
                    // The updates that will be performed
                    let mut updates = vec![];

                    // Fetch the element as a group
                    let group_wrapper = match frame.elements.get(&ElementId::Assigned(group_element_id)) {
                        Some(wrapper)   => wrapper,
                        None            => { return (reversed, vec![]); }
                    };

                    // Gather information on where the grouped elements will go
                    let parent          = group_wrapper.parent;
                    let order_after     = group_wrapper.order_after;
                    let order_before    = group_wrapper.order_before;

                    // Fetch the group elements
                    let elements    = match &group_wrapper.element {
                        Vector::Group(group)    => group.elements().map(|elem| elem.id()).collect::<Vec<_>>(),
                        _                       => { return (reversed, vec![]); }
                    };
                    let group_type  = match &group_wrapper.element {
                        Vector::Group(group)    => group.group_type(),
                        _                       => { return (reversed, vec![]); },
                    };

                    // Reverse re-groups the elements
                    if let Some(order_before) = order_before {
                        reversed.push(AnimationEdit::Element(vec![ElementId::Assigned(group_element_id)], ElementEdit::Order(ElementOrdering::Before(order_before))));
                    }
                    reversed.push(AnimationEdit::Element(elements.clone(), ElementEdit::Group(ElementId::Assigned(group_element_id), group_type)));

                    // Unlink all of the elements from the group
                    for elem in elements.iter() {
                        updates.extend(frame.unlink_element(*elem));
                    }

                    // Unlink the group itself
                    updates.extend(frame.unlink_element(ElementId::Assigned(group_element_id)));

                    // Order the elements after the place where the group was
                    for elem in elements.iter().rev() {
                        updates.extend(frame.order_after(*elem, parent, order_after));
                    }

                    // Result is the updates
                    (reversed, updates)
                }.boxed()
            }).await.unwrap();

            self.request(updates).await;

            // Recreate any groups in reverse order
            let mut reversed = reversed;
            reversed.reverse();
            reversed
        }
    }

    ///
    /// Re-orders a set of elements in their keyframes
    ///
    pub fn order_elements<'a>(&'a mut self, element_ids: Vec<i64>, ordering: ElementOrdering) -> impl 'a + Future<Output=ReversedEdits>  {
        async move {
            // List of updates to perform as a result of this ordering operation
            let mut reverse = ReversedEdits::new();
            let mut updates = vec![];

            // Order each element in turn
            for element_id in element_ids {
                // Load the keyframe for this element if none is currently loaded
                let current_keyframe = self.edit_keyframe_for_element(element_id).await;

                // Order the element in the keyframe
                let maybe_updates = current_keyframe.and_then(|keyframe| {
                    keyframe.sync(|keyframe| {
                        let element_id      = ElementId::Assigned(element_id);

                        // Reset the 'before' element to reverse this
                        let wrapper         = keyframe.elements.get(&element_id);
                        if let Some(wrapper) = wrapper {
                            // Move the element to 'before' the element it is currently behind
                            if let Some(order_before) = wrapper.order_before {
                                reverse.push(AnimationEdit::Element(vec![element_id], ElementEdit::Order(ElementOrdering::Before(order_before))));
                            } else {
                                // The topmost element is the one that has 'None' set as order_before
                                reverse.push(AnimationEdit::Element(vec![element_id], ElementEdit::Order(ElementOrdering::ToTop)));
                            }

                            // Reparent the element before re-ordering (note the 'reverse()' later on, which is why we're adding this afterwards here)
                            if let ElementOrdering::WithParent(_new_parent_id) = &ordering {
                                if let Some(old_parent_id) = wrapper.parent {
                                    // Moving from an existing group
                                    reverse.push(AnimationEdit::Element(vec![element_id], ElementEdit::Order(ElementOrdering::WithParent(old_parent_id))));
                                } else {
                                    // Order at the front of the top-level list of elements
                                    reverse.push(AnimationEdit::Element(vec![element_id], ElementEdit::Order(ElementOrdering::ToTopLevel)));
                                }
                            }
                        }

                        // Re-order the element
                        keyframe.order_element(element_id, ordering)
                    })
                });

                // Add the updates
                maybe_updates.map(|keyframe_updates| updates.extend(keyframe_updates));
            }

            // Re-order in reverse order
            reverse.reverse();

            // Perform the updates
            self.request(updates).await;

            reverse
        }
    }
}
