use super::keyframe_core::*;
use super::super::super::traits::*;

use flo_canvas::*;

use std::sync::*;
use std::time::{Duration};

///
/// A frame from a stream animation
///
pub struct StreamFrame {
    /// When this frame exists
    frame_time: Duration,

    /// The keyframe that was retrieved for this frame (or none if no keyframe was retrieved)
    keyframe_core: Option<KeyFrameCore>
}

impl StreamFrame {
    ///
    /// Creates a new stream frame
    ///
    pub (super) fn new(frame_time: Duration, keyframe_core: Option<KeyFrameCore>) -> StreamFrame {
        StreamFrame {
            frame_time:     frame_time,
            keyframe_core:  keyframe_core
        }
    }
}

impl Frame for StreamFrame {
    ///
    /// Time index of this frame relative to its keyframe
    ///
    fn time_index(&self) -> Duration {
        self.frame_time
    }

    ///
    /// Renders this frame to a particular graphics context
    ///
    fn render_to(&self, gc: &mut dyn GraphicsPrimitives) {
        // Set up the properties
        let mut properties          = Arc::new(VectorProperties::default());
        let mut active_attachments  = vec![];
        let when                    = self.time_index();

        // Render the elements
        if let Some(core) = self.keyframe_core.as_ref() {
            // Start at the initial element
            let mut next_element    = core.initial_element;

            while let Some(current_element) = next_element {
                // Fetch the element definition
                let wrapper = core.elements.get(&current_element);
                let wrapper = match wrapper {
                    Some(wrapper)   => wrapper,
                    None            => { break; }
                };

                // Render the element if it is displayed on this frame
                if wrapper.start_time <= self.frame_time {
                    // Check the attachments
                    if active_attachments != wrapper.attachments {
                        // Update the properties based on the new attachments
                        active_attachments = wrapper.attachments.clone();

                        // Apply the properties from each of the attachments in turn
                        properties = Arc::new(VectorProperties::default());
                        for attachment_id in active_attachments.iter() {
                            if let Some(attach_element) = core.elements.get(&attachment_id) {
                                properties = attach_element.element.update_properties(Arc::clone(&properties));
                                properties.render(gc, attach_element.element.clone(), when);
                            }
                        }
                    }

                    // Render the element
                    properties.render(gc, wrapper.element.clone(), when);
                }

                // Move on to the next element in the list
                next_element = wrapper.order_before;
            }
        }
    }

    ///
    /// Applies all of the properties for the specified element (including those added by attached elements)
    ///
    fn apply_properties_for_element(&self, element: &Vector, properties: Arc<VectorProperties>) -> Arc<VectorProperties> {
        if let Some(core) = self.keyframe_core.as_ref() {
            // Ask the core to apply the properties
            core.apply_properties_for_element(element, properties)
        } else {
            // Properties are unaltered
            properties
        }
    }

    ///
    /// Attempts to retrieve the vector elements associated with this frame, if there are any
    ///
    fn vector_elements<'a>(&'a self) -> Option<Box<dyn 'a+Iterator<Item=Vector>>> {
        if let Some(core) = self.keyframe_core.as_ref() {
            let mut result      = vec![];

            // Start at the initial element
            let mut next_element    = core.initial_element;

            while let Some(current_element) = next_element {
                // Fetch the element definition
                let wrapper = core.elements.get(&current_element);
                let wrapper = match wrapper {
                    Some(wrapper)   => wrapper,
                    None            => { break; }
                };

                // Store the element in the result
                if wrapper.start_time <= self.frame_time {
                    result.push(wrapper.element.clone());
                }

                // Move on to the next element in the list
                next_element = wrapper.order_before;
            }

            Some(Box::new(result.into_iter()))
        } else {
            // No elements
            None
        }
    }

    ///
    /// Retrieves a copy of the element with the specifed ID from this frame, if it exists
    ///
    fn element_with_id(&self, id: ElementId) -> Option<Vector> {
        if let Some(core) = self.keyframe_core.as_ref() {
            // Start at the initial element
            core.elements.get(&id).map(|wrapper| wrapper.element.clone())
        } else {
            // No elements
            None
        }
    }

    ///
    /// Retrieves the IDs and types of the elements attached to the element with a particular ID
    ///
    /// (Element data can be retrieved via element_with_id)
    ///
    fn attached_elements(&self, id: ElementId) -> Vec<(ElementId, VectorType)> {
        if let Some(core) = self.keyframe_core.as_ref() {
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
        } else {
            // No elements
            vec![]
        }
    }
}
