use super::element_wrapper::*;
use super::stream_animation_core::*;
use super::pending_storage_change::*;
use crate::undo::*;
use crate::traits::*;
use crate::editor::*;

use flo_curves::bezier::path::*;

use itertools::*;
use futures::prelude::*;

use std::sync::*;
use std::time::{Duration};

///
/// The output of the layer cut operation
///
#[derive(Clone)]
pub (super) struct LayerCut {
    /// The group of elements that are outside of the path (but cut by it), and the element that they replace
    pub outside_path: Vec<(ElementId, ElementWrapper)>,

    /// The group of elements that are inside of the path (created as new elements after the cut operation)
    pub inside_path: Vec<ElementWrapper>,

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
            let layer_cut = frame.future_sync(move |frame| {
                async move {
                    let mut moved_elements      = vec![];
                    let mut inside_path         = vec![];
                    let mut outside_path        = vec![];

                    let mut remaining_elements  = frame.root_elements();
                    remaining_elements.reverse();

                    // Iterate through all the elements in the frame
                    while let Some(current_element_id) = remaining_elements.pop() {
                        // Fetch the wrapper for the current element
                        let current_element     = frame.elements.get(&current_element_id);
                        let current_element     = if let Some(current_element) = current_element { current_element } else { break; };

                        // Update the properties for this element
                        let mut properties      = Arc::new(VectorProperties::default());
                        for attachment_id in current_element.attachments.iter() {
                            let attachment  = frame.elements.get(attachment_id);
                            let attachment  = if let Some(attachment) = attachment { attachment } else { continue; };

                            properties      = attachment.element.update_properties(properties, when);
                        }

                        // If the element is a 'standard' group - ie, not one that's already doing path arithmetic, recurse into it
                        if let Vector::Group(group_element) = &current_element.element {
                            if group_element.group_type() == GroupType::Normal {
                                // Process each element in the group individually (reversed, so we process the bottom-most first)
                                // To deal with grouped elements correctly, the parent value must be set correctly for each element wrapper in the group
                                let elements = group_element.elements().collect::<Vec<_>>();
                                for subelement in elements.into_iter().rev() {
                                    remaining_elements.push(subelement.id());
                                }

                                // Don't process this element as a normal path
                                continue;
                            }

                            // Other group types are converted to paths
                        }

                        // Get the path for this element (for the cut operation, we need the interior points to be removed)
                        let element_path        = current_element.element.to_path(&properties, PathConversion::RemoveInteriorPoints);
                        let element_path        = if let Some(element_path) = element_path { element_path } else { continue; };

                        // One of the paths making up the element must intersect our bounds
                        let intersects_bounds   = element_path.iter()
                            .any(|path| Rect::from(path).overlaps(&bounds));
                        if !intersects_bounds { continue; }

                        // Cut the paths to determine which parts of the element are inside or outside the cut path
                        for path in element_path.iter() {
                            // Try to cut the path
                            let cut = path_cut::<Path>(&path.to_subpaths(), &cut_path, 0.01);

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
                                
                                inside_path.push(inside_element);
                            } else {
                                // Path cut in two: remove the old element and replace with two path elements
                                let mut exterior    = Path::from_paths(&cut.exterior_path);
                                let mut interior    = Path::from_paths(&cut.interior_path);
                                exterior.unapply_transformations(&*properties);
                                interior.unapply_transformations(&*properties);
                                let exterior        = PathElement::new(ElementId::Unassigned, exterior);
                                let interior        = PathElement::new(ElementId::Unassigned, interior);

                                let mut exterior    = current_element.clone_with_element(Vector::Path(exterior), false);
                                let mut interior    = current_element.clone_with_element(Vector::Path(interior), false);

                                exterior.unattached = true;
                                interior.unattached = true;

                                outside_path.push((current_element_id, exterior));
                                inside_path.push(interior);
                            }
                        }
                    }

                    LayerCut {
                        outside_path,
                        inside_path,
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
    pub (super) fn apply_layer_cut<'a>(&'a mut self, layer_id: u64, when: Duration, layer_cut: LayerCut, inside_group_id: ElementId) -> impl 'a + Future<Output=ReversedEdits> {
        async move {
            let mut pending         = PendingStorageChange::new();

            // Break apart the structure
            let moved_elements      = layer_cut.moved_elements;
            let inside_path         = layer_cut.inside_path;
            let outside_path        = layer_cut.outside_path;

            // Fetch the frame that we'll be cutting elements in
            let frame               = self.edit_keyframe(layer_id, when).await;
            let frame               = match frame { Some(frame) => frame, None => { return ReversedEdits::empty(); } };

            // Remove the attachments from the elements that we'll be replacing
            let replaced_ids        = outside_path.iter().map(|(elem_id, _)| elem_id.id()).flatten().collect::<Vec<_>>();

            // The reverse instructions recreate all of the replaced and moved elements
            let revert_element_ids  = outside_path.iter().flat_map(|(elem_id, _)| elem_id.id());
            let revert_element_ids  = revert_element_ids.chain(moved_elements.iter().flat_map(|id| id.id()));
            let revert_element_ids  = revert_element_ids.collect::<Vec<_>>();
            let replaced_wrappers   = self.wrappers_for_elements(revert_element_ids.iter().cloned()).await;
            let revert_order        = revert_element_ids.iter().map(|id| ElementId::Assigned(*id)).collect();
            let revert_order        = ReversedEdits::recreate_order(revert_order, &|id| id.id().and_then(|id| replaced_wrappers.get(&id).cloned()));
            let mut revert_replace  = frame.future_sync(move |frame| {
                async move {
                    let mut revert_replace = ReversedEdits::new();

                    for revert_element_id in revert_order {
                        let wrapper = replaced_wrappers.get(&revert_element_id.id().unwrap()).unwrap();
                        revert_replace.extend(ReversedEdits::with_recreated_wrapper(layer_id, &wrapper, &|id| frame.elements.get(&id).cloned()));
                    }

                    revert_replace
                }.boxed()
            }).await.unwrap();

            // Assign element IDs to the outside elements if needed
            let mut outside_path_with_ids       = vec![];
            for (replaced_element_id, outside_element_wrapper) in outside_path.into_iter() {
                let id = self.assign_element_id(outside_element_wrapper.element.id()).await.id().unwrap();
                outside_path_with_ids.push((replaced_element_id, id, outside_element_wrapper));
            }

            // Delete all of the elements before replacing them
            let delete_ids = outside_path_with_ids.iter().flat_map(|(old_id, new_id, _)| [*old_id, ElementId::Assigned(*new_id)]);
            let delete_ids = delete_ids.chain(moved_elements.iter().cloned());
            let delete_ids = delete_ids.unique().collect::<Vec<ElementId>>();
            revert_replace.push_front(AnimationEdit::Element(delete_ids, ElementEdit::Delete));

            // Add the replacement elements after the elements they replace
            let mut pending = frame.future_sync(move |frame| {
                async move {
                    frame.invalidate();

                    // Iterate through the 'outside' elements in reverse order so they end up in the correct order relative to one another
                    for (replaced_element_id, outside_element_id, outside_element_wrapper) in outside_path_with_ids.into_iter().rev() {
                        // Parent is same as the replaced element's parent
                        let parent_id = frame.elements.get(&replaced_element_id)
                            .and_then(|wrapper| wrapper.parent);

                        // Wrapper starts unlinked
                        let mut outside_element_wrapper         = outside_element_wrapper;
                        outside_element_wrapper.order_before    = None;
                        outside_element_wrapper.order_after     = None;
                        outside_element_wrapper.parent          = None;

                        // Add the element to the frame
                        pending.push_element(outside_element_id, outside_element_wrapper.clone());
                        pending.push(StorageCommand::AttachElementToLayer(layer_id, outside_element_id, when));

                        frame.elements.insert(ElementId::Assigned(outside_element_id), outside_element_wrapper);

                        // Order after the replaced element
                        pending.extend(frame.order_after(ElementId::Assigned(outside_element_id), parent_id, Some(replaced_element_id)));

                        // Update the attachments for this element
                        pending.extend(frame.update_attachments(ElementId::Assigned(outside_element_id)));

                        // Remove any attachments for the element that's being replaced (it will be unlinked and deleted later)
                        pending.extend(frame.set_attachments(replaced_element_id, vec![]));
                    }

                    pending
                }.boxed()
            }).await.unwrap();

            // Unlink the moved and removed elements
            let mut pending     = frame.future_sync(move |frame| {
                async move {
                    let replaced_elements = replaced_ids.into_iter()
                        .map(|id| ElementId::Assigned(id))
                        .collect::<Vec<_>>();

                    // Unlink all the elements in the replaced and moved lists
                    for unlink_element_id in replaced_elements.iter().chain(moved_elements.iter()) {
                        pending.extend(frame.unlink_element(*unlink_element_id));
                    }

                    // Delete all the elements in the replaced list
                    for delete_element_id in replaced_elements.iter().map(|elem_id| elem_id.id()).flatten() {
                        frame.elements.remove(&ElementId::Assigned(delete_element_id));
                        pending.push(StorageCommand::DeleteElement(delete_element_id));
                    }

                    pending
                }.boxed()
            }).await.unwrap();

            // Create the inside elements (all those without an assigned element ID)
            let mut updated_elements    = vec![];
            let mut inside_group        = vec![];
            let mut inside_element_ids  = vec![inside_group_id];
            let mut inside_when         = when;
            for mut inside_element_wrapper in inside_path {
                // Assign an ID to this element
                let id = if let ElementId::Assigned(id) = inside_element_wrapper.element.id() {
                    id
                } else {
                    self.assign_element_id(ElementId::Unassigned).await.id().unwrap()
                };
                inside_element_ids.push(ElementId::Assigned(id));

                // Add the element to the group
                inside_element_wrapper.element.set_id(ElementId::Assigned(id));
                let group_element = inside_element_wrapper.element.clone();
                inside_group.push(group_element);

                // Add this element to the pending list
                let when = inside_element_wrapper.start_time;
                updated_elements.push(inside_element_wrapper.clone());
                pending.push_element(id, inside_element_wrapper);
                pending.push(StorageCommand::AttachElementToLayer(layer_id, id, when));

                inside_when = inside_when.min(when);
            }

            // Update the elements in the core
            let updates = frame.future_sync(move |frame| {
                async move {
                    let mut updates = PendingStorageChange::new();

                    for wrapper in updated_elements {
                        let element_id = wrapper.element.id();
                        frame.elements.insert(element_id, wrapper);

                        updates.extend(frame.update_attachments(element_id));
                    }
                    frame.invalidate();

                    updates
                }.boxed()
            }).await.unwrap();
            pending.extend(updates);

            // Put the inside and outside elements into groups, and add those groups to the layer
            let inside_group_id     = self.assign_element_id(inside_group_id).await;
            let inside_group_len    = inside_group.len();

            let inside_group        = GroupElement::new(inside_group_id, GroupType::Normal, Arc::new(inside_group));
            let inside_group        = ElementWrapper::attached_with_element(Vector::Group(inside_group), inside_when);

            let revert_inside_group = ReversedEdits::with_edit(AnimationEdit::Element(inside_element_ids, ElementEdit::Delete));

            // Add the two groups to the frame
            let pending = frame.future_sync(move |frame| {
                async move {
                    if inside_group_len > 0 { 
                        pending.extend(frame.add_element_to_end(inside_group.element.id(), inside_group));
                    }

                    pending
                }.boxed()
            }).await.unwrap();

            // Apply the pending storage changes
            self.request(pending).await;

            // Reversion is to delete the group then recreate the elements
            ReversedEdits::with_edits(revert_inside_group.into_iter().chain(revert_replace))
        }
    }
}
