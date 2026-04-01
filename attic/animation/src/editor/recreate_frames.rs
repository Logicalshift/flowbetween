use crate::undo::*;
use crate::traits::*;
use crate::storage::*;

use futures::prelude::*;

use std::time::{Duration};
use std::collections::{HashSet};

impl ReversedEdits {
    ///
    /// Returns the edits required to regenerate an entire keyframe
    ///
    /// The existing elements hashset contains the elements that are already available but not connected to the specified keyframe
    ///
    pub (crate) fn with_recreated_keyframe<'a>(layer_id: u64, keyframe: Duration, existing_elements: &'a mut HashSet<ElementId>, storage_connection: &'a mut StorageConnection) -> impl 'a + Future<Output=ReversedEdits> {
        async move {
            // Start by creating the keyframe on the layer
            let mut recreate_keyframe = ReversedEdits::with_edit(AnimationEdit::Layer(layer_id, LayerEdit::AddKeyFrame(keyframe)));

            // Fetch all of the elements for the specified keyframe from the storage layer
            let keyframe_content = storage_connection.read_keyframe(layer_id, keyframe).await;
            let keyframe_content = if let Some(keyframe_content) = keyframe_content { keyframe_content } else { return recreate_keyframe; };

            // TODO: anything in existing_elements needs to be attached to the keyframe instead of created (right now we're just ignoring them)

            // Recreate each of the elements in the keyframe
            // Elements need to be created after their 'order_before' element
            // Attachments need to be created ahead of the element they're attached to
            // Groups need be created before the elements that they group (and can't be created by the CreateElement call)
            let mut pending_elements = keyframe_content.element_ids.iter().cloned().collect::<Vec<_>>();

            while let Some(next_element) = pending_elements.pop() {
                // Do nothing if the element is already created
                if existing_elements.contains(&next_element) { continue; }

                // Fetch the wrapper for the next element
                let wrapper = keyframe_content.elements.get(&next_element);
                let wrapper = if let Some(wrapper) = wrapper { wrapper } else { continue; };

                if let Some(order_after) = wrapper.order_after {
                    // If this element has an 'order_after', then create that first, then this new element
                    if !existing_elements.contains(&order_after) {
                        pending_elements.push(next_element);
                        pending_elements.push(order_after);
                        continue;
                    }
                }

                if wrapper.attachments.iter().any(|attach_id| !existing_elements.contains(attach_id)) {
                    // Process any missing attachments before the element they're attached to
                    pending_elements.push(next_element);
                    pending_elements.extend(wrapper.attachments.clone());
                    continue;
                }

                if let Vector::Group(group) = &wrapper.element {
                    let sub_element_ids = group.elements().map(|elem| elem.id()).collect::<Vec<_>>();

                    // Groups need all of their child elements created first
                    if sub_element_ids.iter().any(|id| !existing_elements.contains(id)) {
                        // Create the sub-elements of this group before creating the group
                        pending_elements.push(next_element);
                        pending_elements.extend(sub_element_ids);
                    } else {
                        // All the sub-elements exist: recreate the group
                        // This will add the group to the end of the list of attached elements (but as we should regroup everything up to the
                        // top level this should eventually create the original structure)
                        existing_elements.insert(next_element);

                        let group_type  = group.group_type();
                        let regroup     = AnimationEdit::Element(sub_element_ids, ElementEdit::Group(next_element, group_type));

                        recreate_keyframe.push(regroup);
                    }
                } else if wrapper.unattached {
                    // Unattached elements can be created in any order (if they're not otherwise needed)
                    existing_elements.insert(next_element);
                    recreate_keyframe.push(AnimationEdit::Layer(layer_id, LayerEdit::CreateElementUnattachedToFrame(wrapper.start_time, next_element, wrapper.element.clone())));
                } else {
                    // Attached elements are created in order, so we don't need to re-order them
                    existing_elements.insert(next_element);
                    recreate_keyframe.push(AnimationEdit::Layer(layer_id, LayerEdit::CreateElement(wrapper.start_time, next_element, wrapper.element.clone())));
                }

                if !wrapper.attachments.is_empty() {
                    recreate_keyframe.push(AnimationEdit::Element(wrapper.attachments.clone(), ElementEdit::AttachTo(next_element)));
                }
            }

            recreate_keyframe
        }
    }

    ///
    /// Returns the edits required to recreate a whole layer
    ///
    pub (crate) fn with_recreated_layer<'a>(layer_id: u64, storage_connection: &'a mut StorageConnection) -> impl 'a + Future<Output=ReversedEdits> {
        async move {
            // Find the layer to recreate by querying the storage
            let all_layer_ids   = storage_connection.read_layer_ids().await;
            let layer_idx       = all_layer_ids.iter()
                .enumerate()
                .filter(|(_idx, other_layer_id)| *other_layer_id == &layer_id)
                .map(|(idx, _layer_id)| idx)
                .next();
            let layer_idx   = if let Some(layer_idx) = layer_idx { layer_idx } else { return ReversedEdits::empty() };

            if all_layer_ids.len() == 0 { return ReversedEdits::empty(); }

            // Start by recreating the layer and setting its properties
            let mut recreate_layer = ReversedEdits::with_edit(AnimationEdit::AddNewLayer(layer_id));

            if let Some(layer_properties) = storage_connection.read_layer_properties(layer_id).await {
                recreate_layer.push(AnimationEdit::Layer(layer_id, LayerEdit::SetName(layer_properties.name)));
                recreate_layer.push(AnimationEdit::Layer(layer_id, LayerEdit::SetAlpha(layer_properties.alpha)));
            }

            // Order it relative to other layers
            if layer_idx < all_layer_ids.len()-1 {
                // Note that layers are added to the end of this list by default
                let order_before_layer_id = all_layer_ids[layer_idx + 1];
                recreate_layer.push(AnimationEdit::Layer(layer_id, LayerEdit::SetOrdering(order_before_layer_id)));
            }

            // Fetch the keyframes for this layer
            let forever     = Duration::from_millis(0)..Duration::from_micros(i64::MAX as u64);
            let keyframes   = storage_connection.read_keyframes_for_layer(layer_id, forever).await;
            let keyframes   = if let Some(keyframes) = keyframes { keyframes } else { return recreate_layer; };

            // Recreate each keyframe in turn
            let mut created_elements = HashSet::new();
            for keyframe in keyframes {
                let recreate_keyframe = Self::with_recreated_keyframe(layer_id, keyframe.start, &mut created_elements, storage_connection).await;
                recreate_layer.extend(recreate_keyframe);
            }

            // Return the instructions to recreate the layer
            recreate_layer
        }        
    }
}
