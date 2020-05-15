use super::element_align::*;

///
/// Transformations that can be applied to elements
///
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum ElementTransform {
    /// Sets the anchor position for the following transformations (if no anchor is set, the middle of the selection's bounding box is used)
    SetAnchor(f64, f64),

    /// Moves the components to a new location (the specified location is the new location of the anchor: further operations will use this as the anchor position)
    MoveTo(f64, f64),

    /// Aligns several elements to the anchor point (or the overall bounding box)
    Align(ElementAlign),

    /// Flips the element horizontally around the anchor
    FlipHorizontal,

    /// Flips the element vertically around the anchor
    FlipVertical,

    /// Scales the element around the anchor
    Scale(f64, f64),

    /// Rotates the element around the anchor
    Rotate(f64)
}
