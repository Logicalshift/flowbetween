use super::element_wrapper::*;
use crate::undo::*;
use crate::traits::*;

use std::iter;

impl ReversedEdits {
    ///
    /// Restores the links for an element wrapper
    ///
    pub fn with_relinked_element(_layer_id: u64, wrapper: &ElementWrapper, _wrapper_for_element: &impl Fn(ElementId) -> Option<ElementWrapper>) -> ReversedEdits {
        let mut reversed = Self::new();

        // Move to its original parent object
        if let Some(parent) = wrapper.parent {
            // Move into the parent
            reversed.push(AnimationEdit::Element(vec![wrapper.element.id()], ElementEdit::Order(ElementOrdering::WithParent(parent))));

            // TODO: re-order within the parent element
        } else if let Some(order_before) = wrapper.order_before {
            // No parent, but also not the topmost element
            reversed.push(AnimationEdit::Element(vec![wrapper.element.id()], ElementEdit::Order(ElementOrdering::Before(order_before))));
        }

        reversed
    }

    ///
    /// Recreates an element wrapper
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
