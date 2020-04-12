use super::element_id::*;
use super::super::path::*;
use super::super::group_type::*;

use std::sync::*;

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

    /// Updates the control points for this element
    SetControlPoints(Vec<(f32, f32)>),

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
    /// The combined elements will be given the ID of the element that 'bound' to them (ie, from the edit element list)
    /// If there are multiple elements being edited and they bind to each other, the earlier element is the one that
    /// keeps its ID.
    CollideWithExistingElements,

    /// Converts this element to a path
    ConvertToPath
}
