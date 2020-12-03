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
    /// Returns a path component that has been translated by the specified vector
    ///
    pub fn translate(&self, dx: f64, dy: f64) -> PathComponent {
        use self::PathComponent::*;

        match self {
            Close               => Close,
            Move(point)         => Move(point.translate(dx, dy)),
            Line(point)         => Line(point.translate(dx, dy)),
            Bezier(p1, p2, p3)  => Bezier(p1.translate(dx, dy), p2.translate(dx, dy), p3.translate(dx, dy)),
        }
    }

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
