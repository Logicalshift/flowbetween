use super::element_id::*;

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

    /// Updates the control points for this element
    SetControlPoints(Vec<(f32, f32)>),

    /// Updates how this element is ordered relative to other elements in the same keyframe
    /// Note that new elements are always created 'in front' of the current set of elements in the frame.
    Order(ElementOrdering),

    /// Removes this element entirely from the frame
    Delete
}
