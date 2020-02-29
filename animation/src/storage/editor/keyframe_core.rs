use super::stream_animation_core::*;
use super::element_wrapper::*;
use super::super::storage_api::*;
use super::super::super::traits::*;
use super::super::super::serializer::*;

use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashSet, HashMap};

///
/// The keyframe core represents the elements in a keyframe in a particular layer
///
#[derive(Clone)]
pub (super) struct KeyFrameCore {
    /// The ID of the layer that this keyframe is for
    pub (super) layer_id: u64,

    /// The elements in this keyframe
    pub (super) elements: Arc<Mutex<HashMap<ElementId, ElementWrapper>>>,

    /// The first element in the keyframe
    pub (super) initial_element: Option<ElementId>,

    /// The last element in the keyframe
    pub (super) last_element: Option<ElementId>,

    /// The start time of this keyframe
    pub (super) start: Duration,

    /// The end time of this keyframe
    pub (super) end: Duration,

    /// The brush that's active on the last_element, or none if this has not been calculated yet
    pub (super) active_brush: Option<Arc<dyn Brush>>
}

///
/// Resolves an element from a partially resolved list of elements
///
fn resolve_element<'a, Resolver>(unresolved: &mut HashMap<ElementId, Option<Resolver>>, resolved: &'a mut HashMap<ElementId, ElementWrapper>, element_id: ElementId) -> Option<ElementWrapper> 
where Resolver: ResolveElements<ElementWrapper> {
    if let Some(resolved_element) = resolved.get(&element_id) {
        // Already resolved
        Some(resolved_element.clone())
    } else if let Some(Some(unresolved_element)) = unresolved.remove(&element_id) {
        // Exists but is not yet resolved (need to resolve recursively)
        let resolved_element = unresolved_element.resolve(&mut |element_id| { 
            resolve_element(unresolved, resolved, element_id)
                .map(|resolved| resolved.element.clone())
            });

        if let Some(resolved_element) = resolved_element {
            // Resolved the element: add to the resolved list, and return a reference
            resolved.insert(element_id, resolved_element);
            resolved.get(&element_id).cloned()
        } else {
            // Failed to resolve this element
            resolved.insert(element_id, ElementWrapper::error());
            resolved.get(&element_id).cloned()
        }
    } else {
        // Not found
        None
    }
}

impl KeyFrameCore {
    ///
    /// Generates a keyframe by querying the animation core
    ///
    pub fn from_keyframe<'a>(core: &'a mut StreamAnimationCore, layer_id: u64, frame: Duration) -> impl 'a+Future<Output=Option<KeyFrameCore>> {
        async move {
            // Request the keyframe from the core
            let responses = core.request(vec![StorageCommand::ReadElementsForKeyFrame(layer_id, frame)]).await.unwrap_or_else(|| vec![]);

            // Deserialize the elements for the keyframe
            let mut element_ids = vec![];
            let mut elements    = HashMap::new();
            let mut start_time  = frame;
            let mut end_time    = frame;

            for response in responses {
                use self::StorageResponse::*;

                match response {
                    NotFound                            => { return None; }
                    KeyFrame(start, end)                => { start_time = start; end_time = end; }
                    Element(element_id, serialized)     => {
                        // Add the element to the list we know about for this keyframe
                        let element_id  = ElementId::Assigned(element_id);
                        let element     = ElementWrapper::deserialize(element_id, &mut serialized.chars());

                        elements.insert(element_id, element);
                        element_ids.push(element_id);
                    }

                    _                                   => { }
                }
            }

            // Attempt to resolve the elements (missing elements will be changed to error elements)
            let mut resolved = HashMap::<ElementId, ElementWrapper>::new();

            for element_id in element_ids.iter() {
                if let Some(resolver) = elements.remove(element_id) {
                    // Element needs to be resolved
                    if let Some(resolver) = resolver {
                        // Attempt to resolve this element using the others that are attached to this keyframe
                        let resolved_element = resolver.resolve(&mut |element_id| {
                            resolve_element(&mut elements, &mut resolved, element_id)
                                .map(|resolved| resolved.element.clone())
                        });

                        // Store the resolved element
                        if let Some(resolved_element) = resolved_element {
                            resolved.insert(*element_id, resolved_element);
                        }
                    } else {
                        // Element cannot be resolved
                        resolved.insert(*element_id, ElementWrapper::error());
                    }
                } else {
                    // Already resolved this element so there's nothing more to do
                }
            }
            
            // The initial element is the first element we can find with no parent and not ordered after any element
            // There may be more than one of these: we pick the first in the order that the elements are found
            let mut initial_element = None;

            for element_id in element_ids.iter() {
                if let Some(element_wrapper) = resolved.get(element_id) {
                    if element_wrapper.parent.is_none() && element_wrapper.order_after.is_none() && !element_wrapper.unattached {
                        initial_element = Some(*element_id);
                        break;
                    }
                }
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
                elements:           Arc::new(Mutex::new(resolved)),
                initial_element:    initial_element,
                last_element:       last_element,
                start:              start_time,
                end:                end_time,
                active_brush:       None
            })
        }
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
        let elements            = self.elements.lock().unwrap();
        let mut properties      = Arc::new(VectorProperties::default());
        let mut next_element    = self.initial_element;

        while let Some(element_id) = next_element {
            if let Some(element) = elements.get(&element_id) {
                properties      = element.element.update_properties(properties);
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
    pub fn add_element_to_end(&mut self, new_element_id: ElementId, mut new_element: ElementWrapper) -> Vec<StorageCommand> {
        let last_element        = self.last_element;
        let new_id              = new_element_id.id().unwrap_or(0);

        new_element.order_after = last_element;

        // Some elements cause other effects to the status of the keyframe
        match new_element.element {
            Vector::BrushProperties(_) | Vector::BrushDefinition(_) => { self.active_brush = None; }

            _ => { }
        }

        // Serialize it
        let mut serialized  = String::new();
        new_element.serialize(&mut serialized);

        if !new_element.unattached {
            // Add to the current keyframe as the new last element
            let mut keyframe_elements = self.elements.lock().unwrap();
            keyframe_elements.insert(ElementId::Assigned(new_id), new_element.clone());

            let previous_element = last_element.and_then(|last_element| keyframe_elements.get_mut(&last_element));
            let previous_element = if let Some(previous_element) = previous_element {
                previous_element.order_before = Some(ElementId::Assigned(new_id));
                Some(previous_element.clone())
            } else {
                None
            };

            // Update the last element
            self.last_element = Some(ElementId::Assigned(new_id));

            // Generate the storage commands
            if let Some(previous_element) = previous_element {
                // Need to update the previous element as well as the current one
                let previous_element_id             = last_element.and_then(|elem| elem.id()).unwrap_or(0);
                let mut previous_elem_serialized    = String::new();
                
                previous_element.serialize(&mut previous_elem_serialized);

                vec![StorageCommand::WriteElement(previous_element_id, previous_elem_serialized), StorageCommand::WriteElement(new_id, serialized), StorageCommand::AttachElementToLayer(self.layer_id, new_id, new_element.start_time)]
            } else {
                // Just creating a new element
                vec![StorageCommand::WriteElement(new_id, serialized), StorageCommand::AttachElementToLayer(self.layer_id, new_id, new_element.start_time)]
            }
        } else {
            // Unattached elements are just attached to the layer without updating the position
            vec![StorageCommand::WriteElement(new_id, serialized), StorageCommand::AttachElementToLayer(self.layer_id, new_id, new_element.start_time)]
        }
    }

    ///
    /// Attempts to re-order an element relative to the others in the keyframe, returning the storage commands needed to update the underlying storage
    /// 
    /// Returns none if the element is not in the current keyframe
    ///
    pub fn order_element(&mut self, element_id: ElementId, ordering: ElementOrdering) -> Option<Vec<StorageCommand>> {
        let mut elements = self.elements.lock().unwrap();

        if elements.contains_key(&element_id) {
            // Update the element
            let mut updates         = vec![];
            let mut edit_elements   = vec![];

            // Fetch the element (we know it exists at this point)
            let mut element         = elements.get(&element_id).unwrap().clone();

            // Order the element
            use self::ElementOrdering::*;
            match ordering {
                InFront         => {
                    // Fetch the element that's in front of the current element
                    let element_id_in_front     = element.order_before;
                    let element_id_behind       = element.order_after;
                    let element_in_front        = element.order_before.and_then(|order_before| elements.get(&order_before).cloned());
                    let mut element_behind      = element.order_after.and_then(|order_after| elements.get(&order_after).cloned());

                    if let Some(mut element_in_front) = element_in_front {
                        // This element becomes the 'in front' element if it's ahead of all the others
                        if element_id_in_front == self.last_element {
                            self.last_element = Some(element_id);
                        }

                        // Swap the in-front element and the before element
                        element.order_before                                        = element_in_front.order_before;
                        element.order_after                                         = element_id_in_front;
                        element_in_front.order_before                               = Some(element_id);
                        element_in_front.order_after                                = element_id_behind;
                        element_behind.as_mut().map(|behind| behind.order_before    = element_id_in_front);

                        // These are the two elements that need updating
                        edit_elements.push((element_id, element));
                        edit_elements.push((element_id_in_front.unwrap(), element_in_front));
                        element_behind.map(|behind| edit_elements.push((element_id_behind.unwrap(), behind)));
                    }
                }

                Behind          => {
                    // Fetch the element that's in front of the current element
                    let element_id_in_front     = element.order_before;
                    let element_id_behind       = element.order_after;
                    let mut element_in_front    = element.order_before.and_then(|order_before| elements.get(&order_before).cloned());
                    let element_behind          = element.order_after.and_then(|order_after| elements.get(&order_after).cloned());

                    if let Some(mut element_behind) = element_behind {
                        // This element becomes the lowermost element if it's replacing the current lower-most element
                        if element_id_behind == self.initial_element {
                            self.initial_element = Some(element_id);
                        }

                        // Swap the in-front element and the before element
                        element.order_after                                             = element_behind.order_after;
                        element.order_before                                            = element_id_behind;
                        element_behind.order_after                                      = Some(element_id);
                        element_behind.order_before                                     = element_id_in_front;
                        element_in_front.as_mut().map(|in_front| in_front.order_after   = element_id_behind);

                        // These are the two elements that need updating
                        edit_elements.push((element_id, element));
                        edit_elements.push((element_id_behind.unwrap(), element_behind));
                        element_in_front.map(|in_front| edit_elements.push((element_id_in_front.unwrap(), in_front)));
                    }
                }

                ToTop           => {
                    // Nothing to do if the element is already on top
                    if element.order_before.is_some() {
                        // Remove the element from the list
                        let element_id_in_front = element.order_before;
                        let element_id_behind   = element.order_after;
                        let element_in_front    = element.order_before.and_then(|order_before| elements.get(&order_before).cloned());
                        let element_behind      = element.order_after.and_then(|order_after| elements.get(&order_after).cloned());

                        if let Some(mut element_behind) = element_behind.clone() {
                            element_behind.order_before = element_id_in_front;
                            edit_elements.push((element_id_behind.unwrap(), element_behind));
                        }

                        if let Some(mut element_in_front) = element_in_front.clone() {
                            element_in_front.order_after = element_id_behind;
                            edit_elements.push((element_id_in_front.unwrap(), element_in_front));
                        }

                        if self.initial_element == Some(element_id) { self.initial_element = element_id_in_front; }

                        // Find the element that's on top (has an empty 'order_before' element, starting at our existing element)
                        let mut on_top_id = element.order_before.unwrap();
                        while let Some(on_top_element) = elements.get(&on_top_id) {
                            if let Some(order_before) = on_top_element.order_before {
                                // Move to the next element along
                                on_top_id = order_before;
                            } else {
                                // Stop when we reach the top-most element
                                break;
                            }
                        }

                        // Edit the element to be the one on top
                        let mut element_on_top      = if Some(on_top_id) == element_id_in_front { element_in_front.unwrap() } else { elements.get_mut(&on_top_id).unwrap().clone() };

                        element.order_before        = None;
                        element.order_after         = Some(on_top_id);
                        element_on_top.order_before = Some(element_id);

                        edit_elements.push((on_top_id, element_on_top));

                        if self.last_element == Some(on_top_id) { self.last_element = Some(element_id); }
                        edit_elements.push((element_id, element));
                    }
                }

                ToBottom        => {
                    // Nothing to do if the element is already on bottom
                    if element.order_before.is_some() {
                        // Remove the element from the list
                        let element_id_in_front = element.order_before;
                        let element_id_behind   = element.order_after;
                        let element_in_front    = element.order_before.and_then(|order_before| elements.get(&order_before).cloned());
                        let element_behind      = element.order_after.and_then(|order_after| elements.get(&order_after).cloned());

                        if let Some(mut element_behind) = element_behind {
                            element_behind.order_before = element_id_in_front;
                            edit_elements.push((element_id_behind.unwrap(), element_behind));
                        }

                        if let Some(mut element_in_front) = element_in_front {
                            element_in_front.order_after = element_id_behind;
                            edit_elements.push((element_id_in_front.unwrap(), element_in_front));
                        }

                        if self.last_element == Some(element_id) { self.last_element = element_id_behind; }

                        // Find the element that's on bottom (has an empty 'order_after' element, starting at our existing element)
                        let mut on_bottom_id = element.order_after.unwrap();
                        while let Some(on_bottom_element) = elements.get(&on_bottom_id) {
                            if let Some(order_after) = on_bottom_element.order_after {
                                // Move to the next element along
                                on_bottom_id = order_after;
                            } else {
                                // Stop when we reach the bottom-most element
                                break;
                            }
                        }

                        // Edit the element to be the one on bottom
                        element.order_before   = Some(on_bottom_id);
                        element.order_after    = None;
                        elements.get_mut(&on_bottom_id).unwrap().order_after = Some(element_id);

                        if self.initial_element == Some(on_bottom_id) { self.initial_element = Some(element_id); }
                        edit_elements.push((element_id, element));
                    }
                }

                Before(elem)    => {
                    unimplemented!()
                }
            }

            // Convert the element edits to updates
            for (changed_element_id, changed_element_wrapper) in edit_elements {
                // Serialize it
                let mut serialized = String::new();
                changed_element_wrapper.serialize(&mut serialized);

                // Add to the elements
                elements.insert(changed_element_id, changed_element_wrapper);

                // Update in the storage
                updates.push(StorageCommand::WriteElement(changed_element_id.id().unwrap(), serialized));
            }

            Some(updates)
        } else {
            // Element not found
            None
        }
    }
}
