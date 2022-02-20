use super::element_wrapper::*;
use crate::undo::*;
use crate::traits::*;

use std::iter;
use std::time::{Duration};

impl ReversedEdits {
    ///
    /// Recreates an element wrapper
    ///
    pub fn with_recreated_wrapper(wrapper: &ElementWrapper) -> ReversedEdits {
        Self::unimplemented()
    }

    ///
    /// Recreates a vector element
    ///
    pub fn with_recreated_element(layer_id: u64, when: Duration, element: &Vector) -> ReversedEdits {
        use self::Vector::*;

        match element {
            // Vectors that directly contain other elements will need those elements to be resolved (which can't be done when deserializing an edit log)
            Transformed(_transform) => { unimplemented!("Transformed elements should not appear in the undo log") }
            Motion(_motion)         => { unimplemented!("Motion elements are deprecated") }
            Error                   => { ReversedEdits::empty() }

            Group(group)            => {
                // Recreate all of the elements in the group
                let sub_elements    = group.elements()
                    .flat_map(|elem| Self::with_recreated_element(layer_id, when, elem).into_iter());

                // Reform into a group
                let element_ids     = group.elements().map(|elem| elem.id()).collect();
                let group_type      = group.group_type();
                let regroup         = AnimationEdit::Element(element_ids, ElementEdit::Group(group.id(), group_type));

                ReversedEdits::with_edits(sub_elements.chain(iter::once(regroup)))
            }

            BrushDefinition(_)      |
            BrushProperties(_)      |
            BrushStroke(_)          |
            Path(_)                 |
            Shape(_)                |
            AnimationRegion(_)      |
            Transformation((_, _))  => {
                ReversedEdits::with_edit(AnimationEdit::Layer(layer_id, LayerEdit::CreateElement(when, element.id(), element.clone())))
            }
        }
    }
}
