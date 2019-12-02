use super::point::*;
use super::component::*;

use flo_curves::bezier::*;

/// A start point and a following element describes a bezier curve
#[derive(Copy, Clone)]
pub struct PathCurve(pub PathPoint, pub PathComponent);

impl Geo for PathCurve {
    type Point = PathPoint;
}

///
/// A point and an element form a bezier curve (ie, a start point and the following element)
///
impl BezierCurveFactory for PathCurve {
    fn from_points(start: PathPoint, (control_point1, control_point2): (PathPoint, PathPoint), end: PathPoint) -> PathCurve {
        PathCurve(start, PathComponent::Bezier(end, control_point1, control_point2))
    }
}

///
/// A point and an element form a bezier curve (ie, a start point and the following element)
///
impl BezierCurve for PathCurve {
    #[inline]
    fn start_point(&self) -> PathPoint {
        let PathCurve(start, _elem) = *self;

        start
    }

    #[inline]
    fn end_point(&self) -> PathPoint {
        use self::PathComponent::*;

        match self {
            &PathCurve(_, Move(point))                  |
            &PathCurve(_, Line(point))                  => point,
            &PathCurve(_, Bezier(point, _cp1, _cp2))    => point,
            &PathCurve(start, Close)                    => start
        }
    }

    #[inline]
    fn control_points(&self) -> (PathPoint, PathPoint) {
        use self::PathComponent::*;

        match self {
            &PathCurve(start, Move(end))            |
            &PathCurve(start, Line(end))            => {
                let distance    = end - start;
                let one_third   = distance * 0.3;

                (start + one_third, end - one_third)
            },
            &PathCurve(_, Bezier(_point, cp1, cp2)) => (cp1, cp2),
            &PathCurve(start, Close)                => (start, start)
        }
    }
}
