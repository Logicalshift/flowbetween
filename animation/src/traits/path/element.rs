use super::point::*;

///
/// Represents an element of a bezier path
/// 
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PathElement {
    Move(PathPoint),
    Line(PathPoint),
    Bezier(PathPoint, PathPoint, PathPoint),
    Close
}
