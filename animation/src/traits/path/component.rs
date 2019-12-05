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

impl PathComponent {
    ///
    /// Returns the number of points a particular path component will use in the database
    ///
    pub fn num_points(&self) -> usize {
        use self::PathComponent::*;

        match self {
            Move(_) | Line(_)   => 1,
            Bezier(_, _, _)     => 3,
            Close               => 1            // No point is stored but a point type is
        }
    }
}
