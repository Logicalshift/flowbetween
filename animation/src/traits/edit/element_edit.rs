use super::element_id::*;
use super::element_transform::*;
use crate::traits::path::*;
use crate::traits::group_type::*;

use std::sync::*;
use std::time::{Duration};

///
/// Possible element ordering operations
///
#[derive(Clone, PartialEq, Debug, Copy)]
pub enum ElementOrdering {
    InFront,
    Behind,
    ToTop,
    ToBottom,
    Before(ElementId)
}

///
/// Represents an edit to an element within a frame
///
#[derive(Clone, PartialEq, Debug)]
pub enum ElementEdit {
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

    /// Detaches elements from any keyframe it's a part of
    DetachFromFrame,

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
    Transform(Vec<ElementTransform>)
}
