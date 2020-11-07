use super::element_wrapper::*;
use super::stream_animation_core::*;
use crate::traits::*;

use flo_curves::bezier::path::*;

use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};

///
/// The output of the layer cut operation
///
pub (super) struct LayerCut {
    /// The group of elements that are outside of the path
    pub outside_path: Option<Vec<ElementWrapper>>,

    /// The group of elements that are inside of the path
    pub inside_path: Option<Vec<ElementWrapper>>,

    /// The elements that should be removed from the layer and replaced with the inside/outside groups
    pub replaced_elements: Vec<ElementId>,

    /// The elements that have been moved into either the inside or outside path
    pub moved_elements: Vec<ElementId>
}

impl LayerCut {
    ///
    /// Returns a layer cut indicating that no elements were matched
    ///
    pub fn empty() -> LayerCut {
        LayerCut {
            outside_path:       None,
            inside_path:        None,
            replaced_elements:  vec![],
            moved_elements:     vec![]
        }
    }
}

impl StreamAnimationCore {
    ///
    /// Splits all the elements in layer that intersect a supplied path into two groups, returning the groups to add and the elements 
    /// to remove in order to perform the split
    ///
    pub (super) fn layer_cut<'a>(&'a mut self, layer_id: u64, when: Duration, path_components: Arc<Vec<PathComponent>>) -> impl 'a+Future<Output=LayerCut> {
        async move {
            // Change the path components into a Path
            let cut_path    = Path::from_elements(path_components.iter().cloned()).to_subpaths();
            let cut_path    = path_remove_overlapped_points::<_, Path>(&cut_path, 0.01);
            let bounds      = Rect::from(&Path::from_paths(&cut_path));

            // Fetch the frame that we'll be cutting elements in
            let frame       = self.edit_keyframe(layer_id, when).await;
            let frame       = match frame { Some(frame) => frame, None => { return LayerCut::empty(); } };

            // Cut the elements that intersect with the path
            let layer_cut = frame.future(move |frame| {
                async move {
                    let mut replaced_elements   = vec![];
                    let mut moved_elements      = vec![];
                    let mut inside_path         = vec![];
                    let mut outside_path        = vec![];

                    let mut next_element_id     = frame.initial_element;
                    let mut properties          = Arc::new(VectorProperties::default());

                    // Iterate through all the elements in the frame
                    while let Some(current_element_id) = next_element_id {
                        // Fetch the wrapper for the current element
                        let current_element     = frame.elements.get(&current_element_id);
                        let current_element     = if let Some(current_element) = current_element { current_element } else { break; };

                        // Update the next element (so we can cut the loop short later on)
                        next_element_id         = current_element.order_before;

                        // Update the properties for this element
                        for attachment_id in current_element.attachments.iter() {
                            let attachment  = frame.elements.get(attachment_id);
                            let attachment  = if let Some(attachment) = attachment { attachment } else { continue; };

                            properties      = attachment.element.update_properties(properties, when);
                        }

                        // TODO: if the element is a 'standard' group - ie, not one that's already doing path arithmetic, recurse into it

                        // Get the path for this element (for the cut operation, we need  the interior points to be removed)
                        let element_path        = current_element.element.to_path(&properties, PathConversion::RemoveInteriorPoints);
                        let element_path        = if let Some(element_path) = element_path { element_path } else { continue; };

                        // One of the paths making up the element must intersect our bounds
                        let intersects_bounds   = element_path.iter()
                            .any(|path| Rect::from(path).overlaps(&bounds));
                        if !intersects_bounds { continue; }

                        // Cut the paths to determine which parts of the element are inside or outside the cut path
                        for path in element_path.iter() {
                            // Try to cut the path
                            let cut = path_cut::<_, _, Path>(&path.to_subpaths(), &cut_path, 0.01);

                            // TODO: deal with the case where there are multiple paths in element_path?
                            if cut.interior_path.len() == 0 {
                                // All elements outside
                                continue;
                            } else if cut.exterior_path.len() == 0 {
                                // All elements inside
                                moved_elements.push(current_element_id);

                                let mut inside_element      = current_element.clone();
                                inside_element.parent       = None;
                                inside_element.order_before = None;
                                inside_element.order_after  = None;
                                inside_path.push(current_element.clone());
                            } else {
                                // Path cut in two: remove the old element and replace with two path elements
                                replaced_elements.push(current_element_id);

                                let exterior    = PathElement::new(ElementId::Unassigned, Path::from_paths(&cut.exterior_path));
                                let interior    = PathElement::new(ElementId::Unassigned, Path::from_paths(&cut.interior_path));

                                let exterior    = current_element.clone_with_element(Vector::Path(exterior), false);
                                let interior    = current_element.clone_with_element(Vector::Path(interior), false);

                                outside_path.push(exterior);
                                inside_path.push(interior);
                            }
                        }
                    }

                    LayerCut {
                        outside_path: Some(outside_path),
                        inside_path: Some(inside_path),
                        replaced_elements,
                        moved_elements
                    }
                }.boxed()
            }).await;

            layer_cut.unwrap()
        }
    }
}
