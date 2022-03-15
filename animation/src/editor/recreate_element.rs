use super::element_wrapper::*;
use crate::undo::*;
use crate::traits::*;

use std::iter;
use std::collections::{HashSet};

impl ReversedEdits {
    ///
    /// Restores the links for an element wrapper
    ///
    pub fn with_relinked_element(_layer_id: u64, wrapper: &ElementWrapper, wrapper_for_element: &impl Fn(ElementId) -> Option<ElementWrapper>) -> ReversedEdits {
        let mut reversed = Self::new();

        // Move to its original parent object
        if let Some(parent) = wrapper.parent {
            // Fetch the parent element and re-order within
            if let Some(parent_wrapper) = wrapper_for_element(parent) {
                // Order within the parent element
                let siblings    = parent_wrapper.element.sub_element_ids();

                let our_id      = wrapper.element.id();
                let our_index   = siblings.iter().enumerate().filter(|(_, id)| *id == &our_id).nth(0);

                if let Some((our_index, _)) = our_index {
                    if our_index >= siblings.len()-1 {
                        // Last item
                        reversed.push(AnimationEdit::Element(vec![wrapper.element.id()], ElementEdit::Order(ElementOrdering::WithParent(parent))));
                    } else {
                        // Order before
                        reversed.push(AnimationEdit::Element(vec![wrapper.element.id()], ElementEdit::Order(ElementOrdering::Before(siblings[our_index+1]))));
                    }
                } else {
                    // Unknown sibling: just move into the parent
                    reversed.push(AnimationEdit::Element(vec![wrapper.element.id()], ElementEdit::Order(ElementOrdering::WithParent(parent))));
                }
            } else {
                // Move into the parent
                reversed.push(AnimationEdit::Element(vec![wrapper.element.id()], ElementEdit::Order(ElementOrdering::WithParent(parent))));
            }
        } else if let Some(order_before) = wrapper.order_before {
            // No parent, but also not the topmost element
            reversed.push(AnimationEdit::Element(vec![wrapper.element.id()], ElementEdit::Order(ElementOrdering::Before(order_before))));
        }

        reversed
    }

    ///
    /// Returns the order that a set of elements should be recreated in to ensure that the resulting elements are put back in the right
    /// order, with all their attachments intact.
    ///
    /// Essentially, the order in which any order before, parent re-ordering and attachment operation will be performed after the
    /// corresponding element has been created.
    ///
    pub fn recreate_order(element_ids: Vec<ElementId>, wrapper_for_element: &impl Fn(ElementId) -> Option<ElementWrapper>) -> Vec<ElementId> {
        // The elements that are deleted/missing before these elements are re-created
        let mut waiting_elements    = element_ids.iter().cloned().collect::<HashSet<_>>();

        // Elements waiting to be processed
        let mut element_ids         = element_ids;

        // The result from the ordering operation
        let mut ordered_elements    = vec![];

        while let Some(next_element) = element_ids.pop() {
            // If the element is already in the result, then carry on (eg: we found it via another element)
            if !waiting_elements.contains(&next_element) { continue; }

            // Fetch the element wrapper
            let next_wrapper = wrapper_for_element(next_element);
            let next_wrapper = if let Some(next_wrapper) = next_wrapper { next_wrapper } else { continue; };

            // If there's a dependency in the waiting elements, process that before carrying on
            if let Some(parent) = next_wrapper.parent {
                if waiting_elements.contains(&parent) {
                    element_ids.push(next_element);
                    element_ids.push(parent);
                    continue;
                }
            }

            if let Some(order_before) = next_wrapper.order_before {
                if waiting_elements.contains(&order_before) {
                    element_ids.push(next_element);
                    element_ids.push(order_before);
                }
            }

            // All the dependencies in the list are already deployed, so deploy this element next
            waiting_elements.remove(&next_element);
            ordered_elements.push(next_element);
        }

        ordered_elements
    }

    ///
    /// Recreates an element wrapper
    ///
    /// This recreates the element, adds it back in order and adds its attachments back again, as well as re-attaching it to anything it was attached to.
    ///
    pub fn with_recreated_wrapper(layer_id: u64, wrapper: &ElementWrapper, wrapper_for_element: &impl Fn(ElementId) -> Option<ElementWrapper>) -> ReversedEdits {
        let mut reversed = Self::new();

        use self::Vector::*;

        // Recreate the element
        match &wrapper.element {
            // Vectors that directly contain other elements will need those elements to be resolved (which can't be done when deserializing an edit log)
            Transformed(_transform) => { unimplemented!("Transformed elements should not appear in the undo log") }
            Motion(_motion)         => { unimplemented!("Motion elements are deprecated") }
            Error                   => { return ReversedEdits::empty(); }

            Group(group)            => {
                // Recreate all of the elements in the group
                let sub_elements    = group.elements()
                    .flat_map(|elem| wrapper_for_element(elem.id()))
                    .flat_map(|elem| Self::with_recreated_wrapper(layer_id, &elem, wrapper_for_element).into_iter());

                // Reform into a group
                let element_ids     = group.elements().map(|elem| elem.id()).collect();
                let group_type      = group.group_type();
                let regroup         = AnimationEdit::Element(element_ids, ElementEdit::Group(group.id(), group_type));

                reversed.extend(sub_elements.chain(iter::once(regroup)))
            }

            BrushDefinition(_)      |
            BrushProperties(_)      |
            BrushStroke(_)          |
            Path(_)                 |
            Shape(_)                |
            AnimationRegion(_)      |
            Transformation((_, _))  => {
                reversed.push(AnimationEdit::Layer(layer_id, LayerEdit::CreateElement(wrapper.start_time, wrapper.element.id(), wrapper.element.clone())))
            }
        }

        // Move to its original parent object
        reversed.extend(Self::with_relinked_element(layer_id, wrapper, wrapper_for_element));

        // Reattach any attachments
        if !wrapper.attachments.is_empty() {
            reversed.push(AnimationEdit::Element(wrapper.attachments.clone(), ElementEdit::AttachTo(wrapper.element.id())));
        }

        // Reattach anywhere it's used
        if !wrapper.attached_to.is_empty() {
            reversed.push(AnimationEdit::Element(wrapper.attached_to.clone(), ElementEdit::AddAttachment(wrapper.element.id())));
        }

        reversed
    }
}
