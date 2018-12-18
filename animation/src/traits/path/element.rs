use super::point::*;

///
/// Represents an element of a bezier path
/// 
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PathComponent {
    /// Move to point
    Move(PathPoint),

    /// Line to point
    Line(PathPoint),

    /// Bezier curve (order is target, control point 1, control point 2)
    Bezier(PathPoint, PathPoint, PathPoint),

    /// Close path
    Close
}
