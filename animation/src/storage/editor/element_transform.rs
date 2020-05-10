use super::core_element::*;
use super::keyframe_core::*;
use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::super::storage_api::*;
use crate::traits::*;

use flo_curves::*;

use futures::prelude::*;
use smallvec::*;

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashMap};

///
/// Structure that represents the transformations made to a set of elements
///
struct TransformsForElements {
    /// The transformations to apply to each element
    transformations_for_element: HashMap<i64, SmallVec<[Transformation; 2]>>,

    /// The element containing the existing transformation for this element (if there is one)
    current_transform_element_id: HashMap<i64, ElementId>
}

impl TransformsForElements {
    ///
    /// Creates a new set of element transformations
    ///
    fn new() -> TransformsForElements {
        TransformsForElements {
            transformations_for_element:    HashMap::new(),
            current_transform_element_id:   HashMap::new()
        }
    }

    ///
    /// Adds the existing transforms for a particular element (if there are any)
    ///
    fn read_transformation(&mut self, id: i64, frame: &KeyFrameCore) {
        // If there are no transformations attached to the element, then it ends up with an empty list
        let mut element_transforms = smallvec![];

        // Fetch the attachments for this element
        let attachments = if let Some(element_wrapper) = frame.elements.get(&ElementId::Assigned(id)) {
            element_wrapper.attachments.clone()
        } else {
            vec![]
        };

        // Try to fetch the transformations for this element from the frame
        for attachment_id in attachments {
            if let Some(attachment_wrapper) = frame.elements.get(&attachment_id) {
                if let Vector::Transformation((_, transform)) = &attachment_wrapper.element {
                    // Add these transforms to the element
                    element_transforms.extend(transform.iter().cloned());
                    self.current_transform_element_id.insert(id, attachment_id);

                    // Only use the first set of transformations if there are multiple (this is the element we'lloverwrite)
                    break;
                }
            }
        }

        // Store the transforms for this element
        self.transformations_for_element.insert(id, element_transforms);
    }

    ///
    /// Requests and applies a transformation for all of the elements in this structure
    ///
    fn transform<'a, TransformFn>(&'a mut self, transform: TransformFn)
    where TransformFn: 'a+Fn(i64) -> SmallVec<[Transformation; 2]> {
        for (id, existing_transform) in self.transformations_for_element.iter_mut() {
            let new_transform = transform(*id);
            existing_transform.extend(new_transform);
        }
    }
}

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
            let mut element_transforms          = TransformsForElements::new();

            for element_id in element_ids.iter() {
                let element_id = *element_id;

                if let Some(frame) = self.edit_keyframe_for_element(element_id).await {
                    // Store the active transformation for this element
                    let element_transforms = &mut element_transforms;
                    frame.sync(move |frame| element_transforms.read_transformation(element_id, frame));

                    // Calculate the origin for this element
                    let bounds = frame.sync(move |frame| Self::bounds_for_element(frame, element_id));

                    // Add to the overall bounding box
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

            // Apply the transformations for each element in turn
            for transform in transformations.iter() {
                match transform {
                    ElementTransform::SetAnchor(x, y)   => { transform_origin = Some(Coord2(*x, *y)); },

                    ElementTransform::MoveTo(x, y)      => { 
                        if let Some(origin) = transform_origin {
                            element_transforms.transform(|_| smallvec![Transformation::Translate(x - origin.x(), y - origin.y())]); 
                        }
                    }
                }
            }

            // Update or create the attachments for each of the elements
            let mut new_attachments = HashMap::new();
            let mut update_elements = vec![];

            for element_id in element_ids.iter() {
                // Get the transforms and the frame for thie element
                if let (Some(new_transformations), Some(frame)) = (element_transforms.transformations_for_element.get(element_id), self.edit_keyframe_for_element(*element_id).await) {
                    if let Some(existing_attachment_id) = element_transforms.current_transform_element_id.get(element_id) {
                        // Replace the transformation element with a new one
                        frame.sync(|frame| {
                            let transform_wrapper       = frame.elements.get_mut(existing_attachment_id)?;
                            let transform_id            = existing_attachment_id.id()?;
                            transform_wrapper.element   = Vector::Transformation((transform_wrapper.element.id(), new_transformations.clone()));

                            update_elements.push(StorageCommand::WriteElement(transform_id, transform_wrapper.serialize_to_string()));

                            Some(())
                        });
                    } else {
                        // Create a new transformation attachment for this element
                        let attachment_id       = self.assign_element_id(ElementId::Unassigned).await;
                        let attachment_wrapper  = ElementWrapper::with_element(Vector::Transformation((attachment_id, new_transformations.clone())), Duration::from_millis(0));

                        update_elements.push(StorageCommand::WriteElement(attachment_id.id().unwrap(), attachment_wrapper.serialize_to_string()));
                        new_attachments.insert(*element_id, attachment_id);
                    }
                }
            }

            // Update any elements that need new attachments
            self.update_elements(new_attachments.keys().cloned().collect::<Vec<_>>(), 
                |wrapper| ElementUpdate::AddAttachments(vec![new_attachments.get(&wrapper.element.id().id().unwrap()).unwrap().clone()])).await;
        }
    }
}
