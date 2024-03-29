use super::element_id::*;
use super::element_transform::*;
use crate::traits::path::*;
use crate::traits::group_type::*;

use flo_canvas_animation::description::*;

use smallvec::*;
use std::sync::*;
use std::time::{Duration};

///
/// Possible element ordering operations
///
#[derive(Clone, PartialEq, Debug, Copy)]
pub enum ElementOrdering {
    /// Moves the element in front of the following element
    InFront,

    /// Moves the element behind the preceding element
    Behind,

    /// Moves the element to the top of the elements owned by its parent element
    ToTop,

    /// Moves the element to the bottom of the elements owned by its parent element
    ToBottom,

    /// Moves the element to before the specified element
    Before(ElementId),

    /// Adds the element to the top of the list of elements owned by the specified parent element (generally a group element)
    WithParent(ElementId),

    /// Moves the element to the front of the top-level element
    ToTopLevel
}

///
/// Represents an edit to an element within a frame
///
#[derive(Clone, PartialEq, Debug)]
pub enum ElementEdit {
    /// Attach this element to another one
    AttachTo(ElementId),

    /// Adds an attachment to this element
    AddAttachment(ElementId),

    /// Removes an attachment from this element
    RemoveAttachment(ElementId),

    /// Updates the control points for this element (at a particular time)
    SetControlPoints(Vec<(f32, f32)>, Duration),

    /// Updates the path for this element
    SetPath(Arc<Vec<PathComponent>>),

    /// Updates how this element is ordered relative to other elements in the same keyframe
    /// Note that new elements are always created 'in front' of the current set of elements in the frame.
    Order(ElementOrdering),

    /// Removes elements entirely from the frame
    Delete,

    /// Combines the specified elements into a Group, removing them from any other group they might already
    /// be in. The group is created with the element ID specified here.
    ///
    /// For groups that involve path arithmetic the properties are taken from the first item in the list. Ordering
    /// within the group is the same as the ordering in the element list.
    Group(ElementId, GroupType),

    /// Any groups in the list are broken into their constituent elements
    Ungroup,

    /// Attempts to join these elements with matching elements in the same frame
    /// 
    /// The ID of the combined element is the same as the ID of the first element of the set. This makes it possible
    /// to call ConvertToPath on the result and get a valid path. There's a slight issue in that this usually generates
    /// a group of the combined elements. The source element will be in this group but will have no ID of its own.
    CollideWithExistingElements,

    /// Converts this element to a path
    ConvertToPath,

    /// Applies one or more transformations to the elements
    Transform(Vec<ElementTransform>),

    /// If this element is an animation element, updates the region description to a new region
    SetAnimationDescription(RegionDescription),

    /// If this element is an animation element, updates the base animation type to a new kind
    SetAnimationBaseType(BaseAnimationType),

    /// If this element is an animation element, adds a new animation effect with a default description
    AddAnimationEffect(SubEffectType),

    /// If this element is an animation element, replaces the subeffect at the specified address with a new description
    /// (Follow the same rules as `EffectDescription::replace_sub_effect()` when the effect is nested: ie, will preserve the nested contents of the effect)
    ReplaceAnimationEffect(Vec<usize>, EffectDescription),
}

impl ElementEdit {
    ///
    /// Retrieves the element IDs used by this edit
    ///
    #[inline]
    pub fn used_element_ids(&self) -> SmallVec<[ElementId; 4]> {
        use ElementEdit::*;

        match self {
            AttachTo(element_id)            => smallvec![*element_id],
            AddAttachment(element_id)       => smallvec![*element_id],
            RemoveAttachment(element_id)    => smallvec![*element_id],
            Group(element_id, _)            => smallvec![*element_id],

            Delete                          => smallvec![],
            Ungroup                         => smallvec![],
            CollideWithExistingElements     => smallvec![],
            ConvertToPath                   => smallvec![],

            SetControlPoints(_, _)          => smallvec![],
            SetPath(_)                      => smallvec![],
            Order(_)                        => smallvec![],
            Transform(_)                    => smallvec![],
            SetAnimationDescription(_)      => smallvec![],
            SetAnimationBaseType(_)         => smallvec![],
            AddAnimationEffect(_)           => smallvec![],
            ReplaceAnimationEffect(_, _)    => smallvec![],
        }
    }
}
