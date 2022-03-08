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

                if let Some(order_before) = wrapper.order_before {
                    // If this element has an 'order_before', then create that first, then this new element
                    if !existing_elements.contains(&order_before) {
                        pending_elements.push(next_element);
                        pending_elements.push(order_before);
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
                    recreate_keyframe.push(AnimationEdit::Layer(layer_id, LayerEdit::CreateElementUnattachedToFrame(keyframe, next_element, wrapper.element.clone())));
                } else {
                    // Attached elements are created in order, so we don't need to re-order them
                    existing_elements.insert(next_element);
                    recreate_keyframe.push(AnimationEdit::Layer(layer_id, LayerEdit::CreateElement(keyframe, next_element, wrapper.element.clone())));
                }
            }

            recreate_keyframe
        }
    }
}
