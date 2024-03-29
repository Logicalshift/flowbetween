use super::stream_animation_core::*;
use super::element_wrapper::*;
use super::pending_storage_change::*;
use crate::storage::*;
use crate::traits::*;

use flo_canvas::*;
use flo_canvas_animation::*;
use ::desync::*;

use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashSet, HashMap};

///
/// The keyframe core represents the elements in a keyframe in a particular layer
///
pub (super) struct KeyFrameCore {
    /// The ID of the layer that this keyframe is for
    pub (super) layer_id: u64,

    /// The elements in this keyframe
    pub (super) elements: HashMap<ElementId, ElementWrapper>,

    /// The first element in the keyframe
    pub (super) initial_element: Option<ElementId>,

    /// The last element in the keyframe
    pub (super) last_element: Option<ElementId>,

    /// The start time of this keyframe
    pub (super) start: Duration,

    /// The end time of this keyframe
    pub (super) end: Duration,

    /// The brush that's active on the last_element, or none if this has not been calculated yet
    pub (super) active_brush: Option<Arc<dyn Brush>>,

    /// The animation layer for this frame (or None if this hasn't been populated yet or has been invalidated)
    pub (super) animation_layer: Mutex<Option<Arc<Desync<AnimationLayer>>>>
}

impl Clone for KeyFrameCore {
    fn clone(&self) -> Self {
        KeyFrameCore {
            layer_id:           self.layer_id,
            elements:           self.elements.clone(),
            initial_element:    self.initial_element,
            last_element:       self.last_element,
            start:              self.start,
            end:                self.end,
            active_brush:       self.active_brush.clone(),
            animation_layer:    Mutex::new(self.animation_layer.lock().unwrap().clone())
        }
    }
}

impl KeyFrameCore {
    ///
    /// Generates a keyframe by querying the animation core
    ///
    pub fn from_keyframe<'a>(core: &'a mut StreamAnimationCore, layer_id: u64, frame: Duration) -> impl 'a+Future<Output=Option<KeyFrameCore>> {
        async move {
            // Request the keyframe from the core
            let keyframe    = core.storage_connection.read_keyframe(layer_id, frame).await?;
            let start_time  = keyframe.start_time;
            let end_time    = keyframe.end_time;
            let resolved    = keyframe.elements;
            let element_ids = keyframe.element_ids;

            // The initial element is the first element we can find with no parent and not ordered after any element
            // There may be more than one of these: we pick the first in the order that the elements are found
            let mut initial_element = None;

            for element_id in element_ids.iter() {
                if let Some(element_wrapper) = resolved.get(element_id) {
                    if element_wrapper.parent.is_none() && element_wrapper.order_after.is_none() && !element_wrapper.unattached {
                        if Some(*element_id) > initial_element {
                            if initial_element.is_some() {
                                warn!("Found more than one possible initial element for this frame: {:?} and {:?}", initial_element.unwrap(), element_id);
                            }

                            // If there are multiple possible initial elements, use the one with the highest element ID so the behaviour is at least consistent
                            initial_element = Some(*element_id);
                        }
                    }
                }
            }

            // In tests, check that there is only one initial element
            #[cfg(test)]
            {
                let mut num_initial_elements = 0;

                for element_id in element_ids.iter() {
                    if let Some(element_wrapper) = resolved.get(element_id) {
                        if element_wrapper.parent.is_none() && element_wrapper.order_after.is_none() && !element_wrapper.unattached {
                            num_initial_elements += 1;
                        }
                    }
                }

                assert!(num_initial_elements <= 1);
            }

            // The final element is found by following the links of 'after' elements from the 'before' element
            let last_element = if let Some(initial_element) = initial_element {
                // Hash set to prevent a bad file from causing us to ente an infinite loop
                let mut visited         = HashSet::new();
                let mut last_element    = initial_element;

                loop {
                    // Stop if we've already seen this element
                    if visited.contains(&last_element) {
                        break;
                    }
                    visited.insert(last_element);

                    // Look up the last element
                    if let Some(element) = resolved.get(&last_element) {
                        // See if it's ordered before another element
                        if let Some(next_element) = element.order_before {
                            // The current 'last' element is ordered before next_element: it becomes the next candidate 'last element'
                            last_element = next_element;
                        } else {
                            // There is no element following this one
                            break;
                        }
                    } else {
                        // Element was not found (treat it as the last element)
                        break;
                    }
                }

                Some(last_element)
            } else {
                // No initial element means no last element
                None
            };

            // Create the keyframe
            Some(KeyFrameCore {
                layer_id:           layer_id,
                elements:           resolved,
                initial_element:    initial_element,
                last_element:       last_element,
                start:              start_time,
                end:                end_time,
                active_brush:       None,
                animation_layer:    Mutex::new(None)
            })
        }
    }

    ///
    /// Invalidates any cached rendering for this frame
    ///
    #[inline]
    pub fn invalidate(&self) {
        (*self.animation_layer.lock().unwrap()) = None;
    }

    ///
    /// Loads the attachments for an element from a core
    ///
    pub fn retrieve_attachments(core: &Arc<KeyFrameCore>, id: ElementId) -> Vec<(ElementId, VectorType)> {
        // Start at the initial element
        if let Some(wrapper) = core.elements.get(&id) {
            // Fetch the types of the attachments to the element
            wrapper.attachments
                .iter()
                .map(|attachment_id| {
                    core.elements.get(attachment_id)
                        .map(|attachment_wrapper| {
                            (*attachment_id, VectorType::from(&attachment_wrapper.element))
                        })
                })
                .flatten()
                .collect()
        } else {
            // Element not found
            vec![]
        }
    }

    ///
    /// Creates the default properties for this frame
    ///
    pub fn default_properties(core: Arc<KeyFrameCore>) -> Arc<VectorProperties> {
        let mut properties  = VectorProperties::default();

        // Retrieve attachments from this frame
        properties.retrieve_attachments = Arc::new(move |element_id| {
            Self::retrieve_attachments(&core, element_id).into_iter()
                .flat_map(|(element_id, _type)| {
                    core.elements.get(&element_id)
                        .map(|wrapper| wrapper.element.clone())
                })
                .collect()
        });

        Arc::new(properties)
    }

    ///
    /// Renders the overlay for this keyframeto an animation layer
    ///
    pub fn render_overlay(core: &Arc<KeyFrameCore>, gc: &mut dyn GraphicsContext, when: Duration) {
        let mut next_element    = core.initial_element;
        let default_properties  = Self::default_properties(Arc::clone(core));

        while let Some(current_element) = next_element {
            // Fetch the wrapper for the element
            let wrapper = core.elements.get(&current_element);
            let wrapper = match wrapper {
                Some(wrapper)   => wrapper,
                None            => { break; }
            };

            // Reset the properties
            let mut properties = Arc::clone(&default_properties);

            // Apply the properties from each of the attachments in turn
            // TODO: more efficient to only work out the properties if they're needed (most elements have no overlay to draw). Need to see if this ever affects performance noticeably though.
            for attachment_id in wrapper.attachments.iter() {
                if let Some(attach_element) = core.elements.get(&attachment_id) {
                    properties = attach_element.element.update_properties(Arc::clone(&properties), when);
                }
            }

            // Render this element
            wrapper.element.render_overlay(gc, &*properties, when);

            // Move on to the next element in the list
            next_element = wrapper.order_before;
        }
    }

    ///
    /// Renders this keyframe to an animation layer
    ///
    pub fn generate_animation_layer(core: &Arc<KeyFrameCore>, layer: &mut AnimationLayer) {
        // Set up the properties
        let mut properties;
        let mut active_attachments  = vec![];
        let mut gc                  = layer.graphics_context();

        // Everything is rendered at t=0 for the purposes of the animation layer
        // TODO: we used to be able to animate by adjusting the 'when' value for different attachments: now we rely entirely on
        // animation layer regions. It might be useful to bring this feature back in the future, but the AnimationLayer structure
        // does not currently support it without recalculating everything every frame.
        let when                    = Duration::from_millis(0);

        // Render the elements
        let default_properties      = Self::default_properties(Arc::clone(core));
        let mut next_element        = core.initial_element;
        let mut current_time        = core.start;

        while let Some(current_element) = next_element {
            // Fetch the element definition
            let wrapper = core.elements.get(&current_element);
            let wrapper = match wrapper {
                Some(wrapper)   => wrapper,
                None            => { break; }
            };

            // Render the element if it is displayed on this frame
            if wrapper.start_time >= core.start {
                // Update the drawing time in the layer
                if current_time != wrapper.start_time {
                    // Update the time in the layer
                    current_time = wrapper.start_time;
                    gc.set_time(wrapper.start_time - core.start);
                }

                // Reset the properties
                properties = Arc::clone(&default_properties);

                // Check the attachments
                if active_attachments != wrapper.attachments {
                    // Update the properties based on the new attachments
                    active_attachments = wrapper.attachments.clone();

                    // Apply the properties from each of the attachments in turn
                    for attachment_id in active_attachments.iter() {
                        if let Some(attach_element) = core.elements.get(&attachment_id) {
                            properties = attach_element.element.update_properties(Arc::clone(&properties), when);
                            properties.render_animated(&mut gc, attach_element.element.clone(), when);
                        }
                    }
                }

                // Render the element
                properties.render_animated(&mut gc, wrapper.element.clone(), when);
            }

            // Move on to the next element in the list
            next_element = wrapper.order_before;
        }
    }

    ///
    /// Retrieves or caches the animation layer for this keyframe
    ///
    pub fn get_animation_layer(core: &Arc<KeyFrameCore>) -> Arc<Desync<AnimationLayer>> {
        let mut maybe_layer = core.animation_layer.lock().unwrap();

        if let Some(layer) = &*maybe_layer {
            // Layer was already created
            Arc::clone(layer)
        } else {
            // Create a new animation layer and start filling it with data
            let new_layer   = Desync::new(AnimationLayer::new());
            let core        = Arc::clone(core);
            new_layer.desync(move |layer| Self::generate_animation_layer(&core, layer));

            // Store the layer
            let new_layer   = Arc::new(new_layer);
            *maybe_layer    = Some(Arc::clone(&new_layer));

            new_layer
        }
    }

    ///
    /// Retrieves the vector elements associated with this frame
    ///
    pub fn vector_elements<'a>(&'a self, frame_time: Duration) -> impl 'a+Iterator<Item=Vector> {
        let mut result          = vec![];

        // Start at the initial element
        let mut next_element    = self.initial_element;

        while let Some(current_element) = next_element {
            // Fetch the element definition
            let wrapper = self.elements.get(&current_element);
            let wrapper = match wrapper {
                Some(wrapper)   => wrapper,
                None            => { break; }
            };

            // Store the element in the result
            if wrapper.start_time <= frame_time {
                result.push(wrapper.element.clone());
            }

            // Move on to the next element in the list
            next_element = wrapper.order_before;
        }

        result.into_iter()
    }

    ///
    /// Retrieves the currently active brush for this keyframe
    ///
    pub fn get_active_brush(&mut self) -> Arc<dyn Brush> {
        if let Some(ref brush) = self.active_brush {
            // Return the cached brush
            return Arc::clone(brush);
        }

        // Calculate a new active brush
        let mut properties      = Arc::new(VectorProperties::default());
        let mut next_element    = self.initial_element;

        while let Some(element_id) = next_element {
            if let Some(element) = self.elements.get(&element_id) {
                properties      = element.element.update_properties(properties, self.start);
                next_element    = element.order_before;
            } else {
                break;
            }
        }

        let active_brush        = Arc::clone(&properties.brush);
        self.active_brush       = Some(Arc::clone(&active_brush));

        active_brush
    }

    ///
    /// Adds an element to the end of this keyframe (as the new last element)
    /// 
    /// Returns the list of storage commands required to update the storage with the new element
    ///
    pub fn add_element_to_end(&mut self, new_element_id: ElementId, mut new_element: ElementWrapper) -> PendingStorageChange {
        let last_element        = self.last_element;
        let new_id              = new_element_id.id().unwrap_or(0);

        new_element.order_after = last_element;

        // Some elements cause other effects to the status of the keyframe
        match new_element.element {
            Vector::BrushProperties(_)  | 
            Vector::BrushDefinition(_)  => { self.active_brush = None; }

            _                           => { }
        }

        // Add to the list of elements in the current frame
        self.elements.insert(ElementId::Assigned(new_id), new_element.clone());
        self.invalidate();

        // Create the updates
        let mut updates     = PendingStorageChange::new();
        let new_start_time  = new_element.start_time;

        if !new_element.unattached {
            // Add to the current keyframe as the new last element
            let previous_element = last_element.and_then(|last_element| self.elements.get_mut(&last_element));
            let previous_element = if let Some(previous_element) = previous_element {
                previous_element.order_before = Some(ElementId::Assigned(new_id));
                Some(previous_element.clone())
            } else {
                None
            };

            // Update the last element
            self.last_element = Some(ElementId::Assigned(new_id));

            // This becomes the initial element if one isn't assigned yet
            if self.initial_element.is_none() {
                self.initial_element = Some(ElementId::Assigned(new_id));
            }

            // Generate the storage commands
            if let Some(previous_element) = previous_element {
                // Need to update the previous element as well as the current one
                let previous_element_id     = last_element.and_then(|elem| elem.id()).unwrap_or(0);

                updates.push_element(previous_element_id, previous_element);
                updates.push_element(new_id, new_element);
                updates.push(StorageCommand::AttachElementToLayer(self.layer_id, new_id, new_start_time));

                updates
            } else {
                // Just creating a new element
                updates.push_element(new_id, new_element);
                updates.push(StorageCommand::AttachElementToLayer(self.layer_id, new_id, new_start_time));

                updates
            }
        } else {
            // Unattached elements are just attached to the layer without updating the position
            updates.push_element(new_id, new_element);
            updates.push(StorageCommand::AttachElementToLayer(self.layer_id, new_id, new_start_time));

            updates
        }
    }

    ///
    /// Returns the root elements for this keyframe, in order
    ///
    pub fn root_elements(&self) -> Vec<ElementId> {
        let mut next_element_id     = self.initial_element;
        let mut root_elements       = vec![];

        while let Some(element_id) = next_element_id {
            root_elements.push(element_id);
            next_element_id = self.elements.get(&element_id).and_then(|wrapper| wrapper.order_before);
        }

        root_elements
    }

    ///
    /// Returns the child element IDs for a parent element
    ///
    pub fn child_elements_for_parent(&self, parent_element_id: ElementId) -> Vec<ElementId> {
        if let Some(parent_wrapper) = self.elements.get(&parent_element_id) {
            parent_wrapper.element.sub_element_ids()
        } else {
            vec![]
        }
    }

    ///
    /// Returns the elements before and after the specified element
    ///
    fn elements_before_and_after(&self, element_id: ElementId) -> (Option<ElementId>, Option<ElementId>) {
        if let Some(wrapper) = self.elements.get(&element_id) {
            // Element found
            if let Some(parent_id) = wrapper.parent {

                // Need to find the before and after by looking at the parent element
                let siblings    = self.child_elements_for_parent(parent_id);
                let idx         = siblings.iter().position(|elem| elem == &element_id);

                match idx {
                    None        => (None, None),
                    Some(idx)   => (
                        if idx > 0 { Some(siblings[idx-1]) } else { None },
                        if idx+1 < siblings.len() { Some(siblings[idx+1]) } else { None }
                    )
                }

            } else {

                // Use the main element ordering
                // Note that 'order_after' means 'order this element after this'
                (wrapper.order_after, wrapper.order_before)

            }
        } else {

            // Element not found, so there's no before or after element
            (None, None)

        }
    }

    ///
    /// Attempts to re-order an element relative to the others in the keyframe, returning the storage commands needed to update the underlying storage
    /// 
    /// Returns none if the element is not in the current keyframe
    ///
    pub fn order_element(&mut self, element_id: ElementId, ordering: ElementOrdering) -> Option<PendingStorageChange> {
        if let Some(element) = self.elements.get(&element_id) {
            // Update the element
            let mut updates         = PendingStorageChange::new();

            // Order the element
            use self::ElementOrdering::*;
            match ordering {
                InFront         => {
                    // We'll order after the element that this element is currently ordered before
                    let element_id_in_front     = self.elements_before_and_after(element_id).1;
                    let parent                  = element.parent;

                    // If we're already the top-most element, there's nothing to do
                    if element_id_in_front.is_some() {
                        // Update the ordering
                        updates.extend(self.order_after(element_id, parent, element_id_in_front));
                    }
                }

                Behind          => {
                    let element             = element.clone();
                    let element_id_behind   = self.elements_before_and_after(element_id).0;
                    let parent              = element.parent;

                    if element_id_behind.is_some() {
                        // We'll order after the element that's behind the element this is currently in front of
                        let element_id_in_front     = element_id_behind.as_ref()
                            .and_then(|behind| self.elements_before_and_after(*behind).0);

                        // Update the ordering
                        updates.extend(self.order_after(element_id, parent, element_id_in_front));
                    }
                }

                ToTop           => {
                    // Follow the links from the element to find the top (relative to this element)
                    let mut topmost_element = element.order_before;
                    let parent              = element.parent;

                    if let Some(parent) = parent {
                        // Order after the last sibling
                        let siblings        = self.child_elements_for_parent(parent);
                        let last_sibling    = siblings.into_iter().last();
                        let last_sibling    = if last_sibling == Some(element_id) { None } else { last_sibling };

                        topmost_element     = last_sibling;

                    } else {

                        // Follow the chain
                        while let Some(next_element) = topmost_element
                            .and_then(|topmost_element| self.elements.get(&topmost_element))
                            .and_then(|topmost| topmost.order_before) {
                            topmost_element = Some(next_element);
                        }
                    }

                    if topmost_element.is_some() {
                        // Order after the topmost element
                        updates.extend(self.order_after(element_id, parent, topmost_element));
                    }
                }

                ToBottom        => {
                    let parent = element.parent;

                    if element.order_after.is_some() || parent.is_some() {
                        // Order to the bottom
                        updates.extend(self.order_after(element_id, parent, None));
                    }
                }

                Before(before_id)    => {
                    // Fetch the 'before' element
                    if self.elements.contains_key(&before_id) {
                        // Unlink the element
                        updates.extend(self.unlink_element(element_id));

                        // We'll order after the element that's behind the element this is currently in front of
                        let before                  = self.elements.get(&before_id).unwrap();
                        let parent                  = before.parent;
                        let element_id_in_front     = self.element_before(before_id);

                        // Update the ordering
                        updates.extend(self.order_after(element_id, parent, element_id_in_front));
                    }
                }

                WithParent(new_parent) => {
                    if let Some(parent_wrapper) = self.elements.get(&new_parent) {
                        // Order after the last element in the group
                        let last_element = parent_wrapper.element.topmost_sub_element();

                        updates.extend(self.order_after(element_id, Some(new_parent), last_element));
                    }
                }

                ToTopLevel => {
                    updates.extend(self.order_after(element_id, None, self.last_element))
                }
            }

            Some(updates)
        } else {
            // Element not found
            None
        }
    }

    ///
    /// Applies all of the properties for the specified element (including those added by attached elements)
    ///
    pub fn apply_properties_for_element(&self, element: &Vector, properties: Arc<VectorProperties>, when: Duration) -> Arc<VectorProperties> {
        // Try to fetch the element from the core
        let wrapper = self.elements.get(&element.id());

        if let Some(wrapper) = wrapper {
            // Apply the attachments from the wrapper
            let mut properties = properties;
            for attachment_id in wrapper.attachments.iter() {
                if let Some(attach_element) = self.elements.get(&attachment_id) {
                    properties = attach_element.element.update_properties(Arc::clone(&properties), when);
                }
            }

            properties
        } else {
            // Element not from this keyframe?
            properties
        }
    }

    ///
    /// Returns the commands needed to add an attachment to the specified element
    ///
    /// If either element does not exist in the frame, this will still generate commands to add the attachment
    /// to the remaining element (on the assumption that we're adding a new element that is not ready to be
    /// edited yet)
    ///
    pub fn add_attachment(&mut self, attach_to: ElementId, attachments: &Vec<ElementId>) -> PendingStorageChange {
        let mut updates = PendingStorageChange::new();

        // Both the thing being attached to and the attachment must have assigned IDs (can't attach unassigned elements)
        if let Some(attach_to_id) = attach_to.id() {
            // Add the attachment to attach_to
            if let Some(attach_to) = self.elements.get_mut(&attach_to) {
                // Only add the attachment if it doesn't already exist
                let mut added = false;
                for attachment in attachments.iter() {
                    if attachment.id().is_some() && !attach_to.attachments.contains(attachment) {
                        // Add the attachment
                        attach_to.attachments.push(*attachment);
                        added = true;
                    }
                }

                // Add to the updates
                if added {
                    updates.push_element(attach_to_id, attach_to.clone());
                }
            }

            // Back-reference from the attachment to attach_to
            for attachment in attachments {
                if let (Some(attachment), Some(attachment_id)) = (self.elements.get_mut(&attachment), attachment.id()) {
                    // Only add the attachment if it doesn't already exist
                    if !attachment.attached_to.contains(&attach_to) {
                        // Add the item that this is attached to
                        attachment.attached_to.push(attach_to);

                        // Add to the updates
                        updates.push_element(attachment_id, attachment.clone());
                    }
                }
            }
        }

        self.invalidate();
        updates
    }

    ///
    /// Sets the attachments for an element in this frame to a new value
    ///
    pub fn set_attachments<ElementIter: IntoIterator<Item=ElementId>>(&mut self, element_id: ElementId, attachments: ElementIter) -> PendingStorageChange {
        let mut updates             = PendingStorageChange::new();

        // Must be used with an assigned element ID
        if !element_id.is_assigned() {
            return updates;
        }

        // Get the element to edit (no action if the element is not in this frame)
        let wrapper                 = self.elements.get_mut(&element_id);
        let wrapper                 = if let Some(wrapper) = wrapper { wrapper } else { return updates; };

        // Fetch the old attachments and work out the new and old sets
        let original_attachments    = wrapper.attachments.iter().cloned().collect::<HashSet<_>>();
        let mut new_attachments     = vec![];
        let mut removed_attachments = original_attachments.clone();

        // Clear the attachments for the element (we'll fill them back in as we go)
        wrapper.attachments         = vec![];

        for attachment_id in attachments {
            if !attachment_id.is_assigned() { continue; }

            // Add to the element attachments
            wrapper.attachments.push(attachment_id);

            // Modify the set of attachments we'll 
            removed_attachments.remove(&attachment_id);
            if !original_attachments.contains(&attachment_id) {
                new_attachments.push(attachment_id);
            }
        }

        if new_attachments.len() == 0 && removed_attachments.len() == 0 {
            // No changes are being made
            return updates;
        }

        // Update the element
        updates.push_element(element_id.id().unwrap(), wrapper.clone());

        // Remove or add the element from the 'attached_to' items for each attachment
        for new_attachment_id in new_attachments {
            if let Some(attachment) = self.elements.get_mut(&new_attachment_id) {
                if !attachment.attached_to.contains(&element_id) {
                    // Not marked as attached to this element
                    attachment.attached_to.push(element_id);
                    updates.push_element(new_attachment_id.id().unwrap(), attachment.clone());
                }
            }
        }

        for removed_attachment_id in removed_attachments {
            if let Some(attachment) = self.elements.get_mut(&removed_attachment_id) {
                if attachment.attached_to.contains(&element_id) {
                    // Needs to be removed from the 'attached to' list for this element
                    attachment.attached_to.retain(|attached_to| attached_to != &element_id);
                    updates.push_element(removed_attachment_id.id().unwrap(), attachment.clone());
                }
            }
        }

        self.invalidate();
        updates
    }

    ///
    /// Ensures that the 'attached_to' value is set correctly for all of the attachments that belong to the specified element
    ///
    pub fn update_attachments(&mut self, element_id: ElementId) -> PendingStorageChange {
        let mut updates     = PendingStorageChange::new();
        let mut attachments = vec![];

        // Get the list of attachments for this element
        if let Some(element_wrapper) = self.elements.get(&element_id) {
            attachments = element_wrapper.attachments.clone();
        }

        // Ensure that the element is included in the 'attached_to' list for each of its attachments
        for attachment_id in attachments {
            // Fetch the attachment
            if let (Some(attachment_wrapper), Some(attachment_id)) = (self.elements.get_mut(&attachment_id), attachment_id.id()) {
                // Check that the element is attached
                if !attachment_wrapper.attached_to.contains(&element_id) {
                    // Add to the list of attachments if it's not
                    attachment_wrapper.attached_to.push(element_id);
                    updates.push_element(attachment_id, attachment_wrapper.clone());
                }
            }
        }

        self.invalidate();
        updates
    }

    ///
    /// Given an element that may contain child items (eg, a group), checks that all the child elements have the
    /// appropriate parent, recursively
    ///
    pub fn update_parents(&mut self, element_id: ElementId) -> PendingStorageChange {
        // TODO: detect (and break?) loops

        // Fetch the element whose attachments will be updated
        let root_element = match self.elements.get(&element_id) {
            Some(element)   => element,
            None            => { return PendingStorageChange::new() }
        };

        // Check for attachments
        let attachments = match &root_element.element {
            Vector::Group(group)    => group.elements().map(|elem| elem.id()).collect::<Vec<_>>(),
            _                       => { return PendingStorageChange::new(); }
        };

        // Update any elements that are out of date
        let mut updates = PendingStorageChange::new();
        for attachment_id in attachments {
            debug_assert!(attachment_id != element_id);

            // Update any element in this child element that does not have its parent set properly
            updates.extend(self.update_parents(attachment_id));

            // Update this element if it does not have its parent set properly
            if let (Some(attachment), Some(attachment_id)) = (self.elements.get_mut(&attachment_id), attachment_id.id()) {
                if attachment.parent != Some(element_id) {
                    // Update the parent
                    attachment.parent = Some(element_id);

                    // Write out the element
                    updates.push_element(attachment_id, attachment.clone());
                }
            }
        }

        // The result is the list of updates
        self.invalidate();
        updates
    }

    ///
    /// Returns the element that comes before the specified element (or None if there is none)
    ///
    pub fn element_before(&self, element_id: ElementId) -> Option<ElementId> {
        // Fetch the element wrapper. Non-existent elements are considered to have no 'before' element
        let wrapper = self.elements.get(&element_id)?;

        if let Some(parent_id) = wrapper.parent {
            // Fetch the sub-elements of the parent element
            let parent_wrapper  = self.elements.get(&parent_id)?;
            let sub_element_ids = parent_wrapper.element.sub_element_ids();

            // Find the index of the element with the specified ID
            let (idx, _)        = sub_element_ids.iter().enumerate().filter(|(_, id)| *id == &element_id).nth(0)?;

            if idx > 0 {
                Some(sub_element_ids[idx-1])
            } else {
                None
            }
        } else {
            // 'order_after' is what this element is ordered after, ie, it's the element that's before this one
            wrapper.order_after
        }
    }

    ///
    /// Returns the element that comes after the specified element (or None if there is none)
    ///
    pub fn element_after(&self, element_id: ElementId) -> Option<ElementId> {
        // Fetch the element wrapper. Non-existent elements are considered to have no 'before' element
        let wrapper = self.elements.get(&element_id)?;

        if let Some(parent_id) = wrapper.parent {
            // Fetch the sub-elements of the parent element
            let parent_wrapper  = self.elements.get(&parent_id)?;
            let sub_element_ids = parent_wrapper.element.sub_element_ids();

            // Find the index of the element with the specified ID
            let (idx, _)        = sub_element_ids.iter().enumerate().filter(|(_, id)| *id == &element_id).nth(0)?;

            if idx < sub_element_ids.len()-1 {
                Some(sub_element_ids[idx+1])
            } else {
                None
            }
        } else {
            // 'order_before' is what this element is ordered before, ie, it's the element that's after this one
            wrapper.order_before
        }
    }

    ///
    /// Adds the specified element so that it appears after the `after` element, with the specified `parent`.
    /// If `after` is None, then the element is inserted at the start. If `parent` is none, the element is added
    /// to the main list for this frame, otherwise it's added to a group.
    ///
    /// The return value is the commands to send to the storage layer to perform this update.
    ///
    pub fn order_after(&mut self, element_id: ElementId, parent: Option<ElementId>, after: Option<ElementId>) -> PendingStorageChange {
        if let Some(parent) = parent {

            // Perform group re-ordering/adding
            self.order_after_in_group(element_id, parent, after)

        } else {

            // Add into the main list for this frame (there is no parent)
            let mut updates = self.unlink_element(element_id);

            let after       = after.and_then(|after| after.id());
            let element_id  = match element_id.id() {
                Some(id)    => id,
                None        => { return PendingStorageChange::new(); }   
            };

            // Update the 'after' element such that it's followed by this element
            let following_element;
            if let Some(after_wrapper) = after.and_then(|after| self.elements.get_mut(&ElementId::Assigned(after))) {
                // The new element is ordered after the 'after' element 
                following_element           = after_wrapper.order_before.and_then(|following| following.id());
                after_wrapper.order_before  = Some(ElementId::Assigned(element_id));

                updates.push_element(after.unwrap(), after_wrapper.clone());
            } else {
                // The new element is ordered at the start
                following_element = self.initial_element.and_then(|elem| elem.id());
            }

            // Update the main element such that it's between the after element and the following element
            if let Some(element_wrapper) = self.elements.get_mut(&ElementId::Assigned(element_id)) {
                // Order relative to the specified element
                element_wrapper.order_after     = after.map(|after| ElementId::Assigned(after));
                element_wrapper.order_before    = following_element.map(|following| ElementId::Assigned(following));
                element_wrapper.unattached      = false;

                updates.push_element(element_id, element_wrapper.clone());

                // Order first if this element is not ordered after any other element
                if after.is_none() {
                    self.initial_element = Some(ElementId::Assigned(element_id));
                }
            }

            // Update the following element so it's after the new element
            if let Some(following_wrapper) = following_element.and_then(|following| self.elements.get_mut(&ElementId::Assigned(following))) {
                // This is ordered after the current element
                following_wrapper.order_after = Some(ElementId::Assigned(element_id));

                updates.push_element(following_element.unwrap(), following_wrapper.clone());
            } else {
                // This becomes the last element
                self.last_element = Some(ElementId::Assigned(element_id));
            }

            self.invalidate();
            updates
        }
    }

    ///
    /// Given an element that is in a group element (the parent), orders/adds it after the 'after' element
    ///
    fn order_after_in_group(&mut self, element_id: ElementId, parent_id: ElementId, after: Option<ElementId>) -> PendingStorageChange {
        // IDs need to be assigned to be editable
        let (element_id, parent_id) = match (element_id.id(), parent_id.id()) {
            (Some(element_id), Some(parent_id)) => (element_id, parent_id),
            _                                   => { return PendingStorageChange::new() }
        };

        // We need a clone of the element we're going to add
        let element_to_add = match self.elements.get(&ElementId::Assigned(element_id)) {
            Some(wrapper_to_add)    => wrapper_to_add.element.clone(),
            None                    => { return PendingStorageChange::new(); }
        };

        // Unlink the existing element (which will also remove it from the group if it's present)
        let mut updates = self.unlink_element(ElementId::Assigned(element_id));

        // Fetch the parent wrapper
        let parent_wrapper = match self.elements.get_mut(&ElementId::Assigned(parent_id)) {
            Some(wrapper)   => wrapper,
            None            => { return updates; }
        };

        match &parent_wrapper.element {
            Vector::Group(group)    => {
                // The parent element is expected to be a group
                let mut group_elements  = group.elements().cloned().collect::<Vec<_>>();
                let after_index         = group_elements.iter().position(|elem| Some(elem.id()) == after);

                // Update the element list
                match after_index {
                    None        => group_elements.insert(0, element_to_add),
                    Some(idx)   => group_elements.insert(idx+1, element_to_add)
                }

                // Update the group
                let new_group               = group.with_elements(group_elements);
                parent_wrapper.element      = Vector::Group(new_group);

                updates.push_element(parent_id, parent_wrapper.clone());

                // Update the element wrapper
                let wrapper_to_add          = self.elements.get_mut(&ElementId::Assigned(element_id)).unwrap();
                wrapper_to_add.parent       = Some(ElementId::Assigned(parent_id));
                wrapper_to_add.unattached   = true;

                updates.push_element(element_id, wrapper_to_add.clone());
            },

            _ => { }
        }

        self.invalidate();
        updates
    }

    ///
    /// Returns true if there are any unlinked elements in this frame
    ///
    pub fn has_unlinked_elements(&self) -> bool {
        // Check that all the root elements have a 'next' element
        let mut next_element    = self.initial_element;
        let mut last_element    = None;

        while let Some(current_element) = next_element {
            // Fetch the wrapper for the element
            let wrapper = self.elements.get(&current_element);
            let wrapper = match wrapper {
                Some(wrapper)   => wrapper,
                None            => { return true; }
            };

            // Move on to the next element in the list
            last_element = Some(current_element);
            next_element = wrapper.order_before;
        }

        // Should wind up at the last element in the frame
        if last_element != self.last_element {
            return true;
        }

        // TODO: check for elements with a 'parent' that's not in the list
        // TODO: check for elements outside of the root set
        // TODO: check for missing attachments
        // TODO: check for attachments added to elements that don't exist

        // None of the tests found any missing elements
        return false;
    }

    ///
    /// Unlinks an element in this frame, and returns the commands required to unlink it
    /// 
    /// This makes it possible to detach or delete this element, or use it in an attachment somewhere else
    /// (eg, when grouping elements)
    ///
    pub fn unlink_element(&mut self, element_id: ElementId) -> PendingStorageChange {
        let element_id_i64  = match element_id.id() { Some(id) => id, None => { return PendingStorageChange::new() } };

        // The updates required to unlink this element
        let mut updates     = PendingStorageChange::new();

        // Fetch this element
        let wrapper         = self.elements.get_mut(&element_id);
        if let Some(wrapper) = wrapper {

            if let Some(parent) = wrapper.parent {

                // Unlink from a group/similar item
                updates = self.unlink_from_group(element_id, parent);

            } else {

                // If this is the initial element the next element becomes the initial element
                if self.initial_element == Some(element_id) {
                    self.initial_element = wrapper.order_before;
                }

                // If this is the last element, the previous element becomes the last element
                if self.last_element == Some(element_id) {
                    self.last_element = wrapper.order_after;
                }

                // We'll need to process the before/after versions next
                let previous_id = wrapper.order_after;
                let next_id     = wrapper.order_before;

                // Make wrapper unattached
                wrapper.unattached      = true;
                wrapper.order_before    = None;
                wrapper.order_after     = None;
                wrapper.parent          = None;

                updates.push_element(element_id_i64, wrapper.clone());

                // Rearrange the previous and next element to skip over this one
                if let Some((Some(id), Some(previous_wrapper))) = previous_id.map(|previous_id| (previous_id.id(), self.elements.get_mut(&previous_id))) {
                    previous_wrapper.order_before = next_id;

                    updates.push_element(id, previous_wrapper.clone());
                }

                if let Some((Some(id), Some(next_wrapper))) = next_id.map(|next_id| (next_id.id(), self.elements.get_mut(&next_id))) {
                    next_wrapper.order_after = previous_id;

                    updates.push_element(id, next_wrapper.clone());
                }

            }
        }

        // Result is the updates to send to the storage layer
        self.invalidate();
        updates
    }

    ///
    /// Unlinks an element from a group
    ///
    pub fn unlink_from_group(&mut self, element_id: ElementId, group_id: ElementId) -> PendingStorageChange {
        let mut updates = PendingStorageChange::new();

        let element_id  = match element_id.id() {
            Some(id)    => id,
            None        => { return updates }
        };
        let group_id    = match group_id.id() {
            Some(id)    => id,
            None        => { return updates }
        };

        // For the element, we just remove the parent from the wrapper
        if let Some(wrapper) = self.elements.get_mut(&ElementId::Assigned(element_id)) {
            wrapper.parent = None;
            updates.push_element(element_id, wrapper.clone());
        }

        // Update the parent group so it doesn't contain this element any more
        if let Some(group_wrapper) = self.elements.get_mut(&ElementId::Assigned(group_id)) {
            match &group_wrapper.element {
                Vector::Group(group) => {
                    // Create a new version of the group that's missing this element
                    let new_group = group.with_elements(group.elements()
                        .filter(|elem| elem.id() != ElementId::Assigned(element_id))
                        .cloned());

                    // Update the wrapper to be missing this element
                    group_wrapper.element = Vector::Group(new_group);
                    updates.push_element(group_id, group_wrapper.clone());
                }

                _ => { }
            }
        }

        self.invalidate();
        updates
    }
}
