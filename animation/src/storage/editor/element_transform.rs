use super::keyframe_core::*;
use super::stream_animation_core::*;
use crate::traits::*;

use flo_curves::*;

use futures::prelude::*;

use std::sync::*;

impl StreamAnimationCore {
    ///
    /// Returns the bounding box for an element
    ///
    fn bounds_for_element(frame: &KeyFrameCore, element_id: i64) -> Option<Rect> {
        if let Some(wrapper) = frame.elements.get(&ElementId::Assigned(element_id)) {
            // Get the properties for this element
            let properties      = frame.apply_properties_for_element(&wrapper.element, Arc::new(VectorProperties::default()), wrapper.start_time);

            // Convert to path
            let paths           = wrapper.element.to_path(&properties, PathConversion::Fastest);

            // Compute the bounding box
            let mut bounding_box: Option<Rect>  = None;
            for path_section in paths.into_iter().flatten() {
                let bounds = path_section.bounding_box();

                bounding_box = if let Some(bounding_box) = bounding_box {
                    Some(bounding_box.union(bounds))
                } else {
                    Some(bounds)
                };
            }

            // Origin is at the center of the path bounds
            bounding_box
        } else {
            // Element does not exist
            None
        }
    }

    ///
    /// Applies transformations to a set of elements
    ///
    pub fn transform_elements<'a>(&'a mut self, element_ids: &'a Vec<i64>, transformations: &'a Vec<ElementTransform>) -> impl 'a+Send+Future<Output=()> {
        async move {
            // Nothing to do if there are no elements
            if element_ids.len() == 0 {
                return;
            }

            // The origin starts as the center point of all of the elments
            let mut bounding_box: Option<Rect>  = None;

            for element_id in element_ids.iter() {
                let element_id = *element_id;

                if let Some(frame) = self.edit_keyframe_for_element(element_id).await {
                    // Calculate the origin for this element
                    let bounds = frame.future(move |frame| {
                        async move {
                            Self::bounds_for_element(frame, element_id)
                        }.boxed()
                    }).await.unwrap();

                    // Add to the sum of the origins
                    if let Some(bounds) = bounds {
                        bounding_box = if let Some(bounding_box) = bounding_box {
                            Some(bounding_box.union(bounds))
                        } else {
                            Some(bounds)
                        }
                    }
                }
            }

            // Set up the initial origin for the transformation
            let mut transform_origin = bounding_box.map(|bounding_box| bounding_box.center());
        }
    }
}