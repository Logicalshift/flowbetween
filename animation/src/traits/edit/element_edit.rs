use super::element_id::*;
use super::super::path::*;

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
    DetachFromFrame
}
