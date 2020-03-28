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

impl StreamAnimationCore {
    ///
    /// Discovers all of the elements in the frame along with their properties
    ///
    async fn frame_elements_with_properties(&self, frame: Arc<Desync<KeyFrameCore>>) -> Vec<(Vector, Arc<VectorProperties>)> {
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
                        result.push((elem.element.clone(), Arc::clone(&current_properties)));
                    }

                    // Move on to the element that's ahead of the current one
                    next_element_id = current_element.and_then(|element| element.order_before);
                }

                result
            }.boxed()
        }).await.unwrap()
    }

    /*
    ///
    /// Attempts to combine an element with other elements in the same frame (by joining them into a single path)
    ///
    pub fn collide_with_existing_elements(&mut self, frame: Arc<dyn Frame>) {
        // We need to know the properties of all of the elements in the current frame (we need to work backwards to generate the grouped element)
        let elements_with_properties        = self.frame_elements_with_properties(frame);
        let brush_points                    = Arc::new(self.current_brush.brush_points_for_raw_points(&self.points));

        // Nothing to do if there are no properties
        if elements_with_properties.len() == 0 {
            return;
        }

        // The vector properties of the brush will be the last element properties with the brush properties added in
        let mut new_properties              = (*elements_with_properties.last().unwrap().1).clone();
        new_properties.brush                = Arc::clone(&self.current_brush);
        new_properties.brush_properties     = self.brush_properties.clone();

        // Attempt to combine the current brush stroke with them
        let mut combined_element            = None;
        for (element, properties) in elements_with_properties.iter().rev() {
            use self::CombineResult::*;

            combined_element = match self.current_brush.combine_with(&element, Arc::clone(&brush_points), &new_properties, &*properties, combined_element.clone()) {
                NewElement(new_combined)    => { Some(new_combined) },
                NoOverlap                   => { continue; },               // Might be able to combine with an element further down
                CannotCombineAndOverlaps    => { break; },                  // Not quite right: we can combine with any element that's not obscured by an existing element (we can skip over overlapping elements we can't combine with)
                UnableToCombineFurther      => { break; }                   // Always stop here
            }
        }

        self.combined_element = combined_element;
    }
    */
}
