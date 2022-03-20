use super::point::*;

use flo_curves::*;

///
/// Represents an element of a bezier path
///
#[derive(Clone, Copy, Debug)]
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

/// 
/// Returns true if two points are 'close enough' to be considered the same in PartialEq
///
fn is_close(a: &PathPoint, b: &PathPoint) -> bool {
    const MIN_DISTANCE: f64 = 0.02;

    let diff    = *a - *b;
    let diff_sq = diff.dot(&diff);

    diff_sq < (MIN_DISTANCE * MIN_DISTANCE)
}

///
/// Two path components are considered to match if their coordinates are 'close enough' (as when we serialize f64 values we compress them in a way that loses a lot of precision)
///
impl PartialEq for PathComponent {
    fn eq(&self, b: &PathComponent) -> bool {
        use PathComponent::*;

        match (self, b) {
            (Move(a), Move(b))                          => is_close(a, b),
            (Line(a), Line(b))                          => is_close(a, b),
            (Bezier(a1, a2, a3), Bezier(b1, b2, b3))    => is_close(a1, b1) && is_close(a2, b2) && is_close(a3, b3),
            (Close, Close)                              => true,

            _ => false,
        }
    }
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
