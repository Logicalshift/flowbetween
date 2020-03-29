use super::keyframe_core::*;
use super::stream_animation_core::*;
use super::super::super::traits::*;

use futures::prelude::*;
use ::desync::*;

use std::sync::*;

impl StreamAnimationCore {
    ///
    /// Discovers all of the elements in the frame along with their properties
    ///
    async fn frame_elements_with_properties(&self, frame: &Arc<Desync<KeyFrameCore>>) -> Vec<(Vector, Arc<VectorProperties>)> {
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

    ///
    /// Attempts to combine an element with other elements in the same frame (by joining them into a single path)
    ///
    pub async fn collide_with_existing_elements(&mut self, combine_element_id: ElementId) {
        // Fetch the frame that this element belongs to
        let assigned_element_id = match combine_element_id.id() {
            Some(id)    => id,
            None        => { return }
        };

        if let Some(frame) = self.edit_keyframe_for_element(assigned_element_id).await {
            // We need to know the properties of all of the elements in the current frame (we need to work backwards to generate the grouped element)
            let elements_with_properties    = self.frame_elements_with_properties(&frame).await;

            // Nothing to do if there are no properties
            if elements_with_properties.len() == 0 {
                return;
            }

            frame.future(move |frame| {
                async move {
                    // Find the brush properties for the selected element. These are usually at the end, so a linear search like this should be fine
                    let new_properties = elements_with_properties.iter().rev()
                        .filter(|elem| elem.0.id() == combine_element_id)
                        .map(|elem| elem.1.clone())
                        .nth(0)
                        .unwrap_or_else(|| Arc::new(VectorProperties::default()));
                    let current_brush = &new_properties.brush;

                    // Fetch the element from the frame
                    let element = frame.elements.get(&combine_element_id);

                    // Collide other elements in the frame with this element
                    // Only brush stroke elements can be combined at the moment
                    match element.map(|wrapper| &wrapper.element) {
                        Some(Vector::BrushStroke(brush_stroke)) => {
                            // Take the brush points from this one to generate a new value for the element
                            let brush_points = brush_stroke.points();

                            // Attempt to combine the element we fetched with the rest of the frame
                            let mut combined_element            = None;
                            for (element, properties) in elements_with_properties.iter().rev() {
                                use self::CombineResult::*;

                                // Ignore the element we're merging
                                // TODO: consider ignoring the elements above the element we're merging too
                                if element.id() == combine_element_id {
                                    continue;
                                }

                                combined_element = match current_brush.combine_with(&element, Arc::clone(&brush_points), &new_properties, &*properties, combined_element.clone()) {
                                    NewElement(new_combined)    => { Some(new_combined) },
                                    NoOverlap                   => { continue; },               // Might be able to combine with an element further down
                                    CannotCombineAndOverlaps    => { break; },                  // Not quite right: we can combine with any element that's not obscured by an existing element (we can skip over overlapping elements we can't combine with)
                                    UnableToCombineFurther      => { break; }                   // Always stop here
                                }
                            }
                        }

                        _ => { }
                    }
                }.boxed()
            }).await.unwrap();
        }

        /*
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
        */
    }
}
