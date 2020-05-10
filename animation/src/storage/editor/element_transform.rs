use super::core_element::*;
use super::keyframe_core::*;
use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::super::storage_api::*;
use crate::traits::*;

use flo_curves::*;

use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashMap};

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

            // The anchor point starts as the center point of all of the elements: calculate this point by computing the bounding box of all the elements
            // This is also used for alignments
            let mut bounding_box: Option<Rect>  = None;
            let mut bounds_for_element          = HashMap::new(); 

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
                        bounds_for_element.insert(element_id, bounds);

                        bounding_box = if let Some(bounding_box) = bounding_box {
                            Some(bounding_box.union(bounds))
                        } else {
                            Some(bounds)
                        };
                    }
                }
            }

            // Set up the initial origin for the transformation
            let mut transform_origin    = bounding_box.map(|bounding_box| bounding_box.center());

            // Build up the transformations to apply to the element
            let mut element_transform   = vec![];
            for transform in transformations.iter() {
                match transform {
                    ElementTransform::SetAnchor(x, y)   => transform_origin = Some(Coord2(*x, *y)),

                    ElementTransform::MoveTo(x, y)      => {
                        if let Some(origin) = transform_origin {
                            element_transform.push(Transformation::Translate(x - origin.x(), y - origin.y()));
                        }
                    }
                }
            }

            // Generate the attachments for these transformations
            let mut new_attachments     = vec![];
            let mut generate_elements   = vec![];
            for transform in element_transform {
                // Create a new wrapper for this transformation
                let attachment_id       = self.assign_element_id(ElementId::Unassigned).await;
                let attachment_wrapper  = ElementWrapper::with_element(Vector::Transformation((attachment_id, transform)), Duration::from_millis(0));

                // Write it out
                generate_elements.push(StorageCommand::WriteElement(attachment_id.id().unwrap(), attachment_wrapper.serialize_to_string()));

                // Attach to the elements
                new_attachments.push(attachment_id);
            }

            self.request(generate_elements).await;

            // Attach to all of the elements
            self.update_elements(element_ids.clone(), |_wrapper| ElementUpdate::AddAttachments(new_attachments.clone())).await
        }
    }
}
