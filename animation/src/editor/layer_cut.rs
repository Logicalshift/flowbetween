use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::pending_storage_change::*;
use crate::traits::*;
use crate::editor::*;

use flo_curves::bezier::path::*;

use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};

///
/// The output of the layer cut operation
///
#[derive(Clone)]
pub (super) struct LayerCut {
    /// The group of elements that are outside of the path
    pub outside_path: Vec<ElementWrapper>,

    /// The group of elements that are inside of the path
    pub inside_path: Vec<ElementWrapper>,

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
            outside_path:       vec![],
            inside_path:        vec![],
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
                                // All elements outside (we can leave these elements alone)
                                continue;
                            } else if cut.exterior_path.len() == 0 {
                                // All elements inside
                                moved_elements.push(current_element_id);

                                let mut inside_element      = current_element.clone();
                                inside_element.parent       = None;
                                inside_element.order_before = None;
                                inside_element.order_after  = None;
                                inside_element.unattached   = true;
                                inside_path.push(current_element.clone());
                            } else {
                                // Path cut in two: remove the old element and replace with two path elements
                                replaced_elements.push(current_element_id);

                                let exterior        = PathElement::new(ElementId::Unassigned, Path::from_paths(&cut.exterior_path));
                                let interior        = PathElement::new(ElementId::Unassigned, Path::from_paths(&cut.interior_path));

                                let mut exterior    = current_element.clone_with_element(Vector::Path(exterior), false);
                                let mut interior    = current_element.clone_with_element(Vector::Path(interior), false);

                                exterior.unattached = true;
                                interior.unattached = true;

                                outside_path.push(exterior);
                                inside_path.push(interior);
                            }
                        }
                    }

                    LayerCut {
                        outside_path,
                        inside_path,
                        replaced_elements,
                        moved_elements
                    }
                }.boxed()
            }).await;

            layer_cut.unwrap()
        }
    }

    ///
    /// Applies a layer cut operation to a frame
    ///
    pub (super) fn apply_layer_cut<'a>(&'a mut self, layer_id: u64, when: Duration, layer_cut: LayerCut, inside_group_id: ElementId, outside_group_id: ElementId) -> impl 'a + Future<Output=()> {
        async move {
            let mut pending         = PendingStorageChange::new();

            // Break apart the structure
            let replaced_elements   = layer_cut.replaced_elements;
            let moved_elements      = layer_cut.moved_elements;
            let inside_path         = layer_cut.inside_path;
            let outside_path        = layer_cut.outside_path;

            // Fetch the frame that we'll be cutting elements in
            let frame           = self.edit_keyframe(layer_id, when).await;
            let frame           = match frame { Some(frame) => frame, None => { return; } };

            let replaced_ids        = replaced_elements.iter().map(|elem_id| elem_id.id()).flatten().collect::<Vec<_>>();
            self.remove_from_attachments(&replaced_ids).await;

            // Unlink the moved and removed elements
            let mut pending     = frame.future(move |frame| {
                async move {
                    // Unlink all the elements in the replaced and moved lists
                    for unlink_element_id in replaced_elements.iter().chain(moved_elements.iter()) {
                        pending.extend(frame.unlink_element(*unlink_element_id));
                    }

                    // Delete all the elements in the replaced list
                    for delete_element_id in replaced_elements.iter().map(|elem_id| elem_id.id()).flatten() {
                        pending.push(StorageCommand::DeleteElement(delete_element_id));
                    }

                    pending
                }.boxed()
            }).await.unwrap();

            // Create the inside elements (all those without an assigned element ID)
            let mut inside_group    = vec![];
            let mut inside_when     = when;
            for mut inside_element_wrapper in inside_path {
                // Assign an ID to this element
                let id = if let ElementId::Assigned(id) = inside_element_wrapper.element.id() {
                    id
                } else {
                    self.assign_element_id(ElementId::Unassigned).await.id().unwrap()
                };

                // Add the element to the group
                inside_element_wrapper.element.set_id(ElementId::Assigned(id));
                let group_element = inside_element_wrapper.element.clone();
                inside_group.push(group_element);

                // Add this element to the pending list
                let when = inside_element_wrapper.start_time;
                pending.push_element(id, inside_element_wrapper);
                pending.push(StorageCommand::AttachElementToLayer(layer_id, id, when));

                inside_when = inside_when.min(when);
            }

            // Create the outside elements (all those without an assigned element ID)
            let mut outside_group   = vec![];
            let mut outside_when    = when;
            for mut outside_element_wrapper in outside_path {
                // Assign an ID to this element
                let id = if let ElementId::Assigned(id) = outside_element_wrapper.element.id() {
                    id
                } else {
                    self.assign_element_id(ElementId::Unassigned).await.id().unwrap()
                };

                // Add the element to the group
                outside_element_wrapper.element.set_id(ElementId::Assigned(id));
                let group_element = outside_element_wrapper.element.clone();
                outside_group.push(group_element);

                // Add this element to the pending list
                let when = outside_element_wrapper.start_time;
                pending.push_element(id, outside_element_wrapper);
                pending.push(StorageCommand::AttachElementToLayer(layer_id, id, when));

                outside_when = outside_when.min(when);
            }

            // Put the inside and outside elements into groups, and add those groups to the layer
            let inside_group_id     = self.assign_element_id(inside_group_id).await;
            let outside_group_id    = self.assign_element_id(outside_group_id).await;
            let inside_group_len    = inside_group.len();
            let outside_group_len   = outside_group.len();

            let inside_group        = GroupElement::new(inside_group_id, GroupType::Normal, Arc::new(inside_group));
            let outside_group       = GroupElement::new(outside_group_id, GroupType::Normal, Arc::new(outside_group));

            let inside_group        = ElementWrapper::attached_with_element(Vector::Group(inside_group), inside_when);
            let outside_group       = ElementWrapper::attached_with_element(Vector::Group(outside_group), outside_when);

            // Add the two groups to the frame
            let pending = frame.future(move |frame| {
                async move {
                    if inside_group_len > 0 { pending.extend(frame.add_element_to_end(inside_group.element.id(), inside_group)); }
                    if outside_group_len > 0 { pending.extend(frame.add_element_to_end(outside_group.element.id(), outside_group)); }

                    pending
                }.boxed()
            }).await.unwrap();

            // Apply the pending storage changes
            self.request(pending).await;
        }
    }
}
