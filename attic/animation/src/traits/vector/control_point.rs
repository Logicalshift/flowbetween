use crate::traits::vector::transformation::*;

use flo_curves::*;

///
/// Represents a control point in a vector element
///
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ControlPoint {
    /// Represents a point on a bezier curve
    BezierPoint(f64, f64),

    /// Represents a bezier control point
    BezierControlPoint(f64, f64)
}

impl ControlPoint {
    ///
    /// Returns the x, y position of this control point
    ///
    pub fn position(&self) -> (f64, f64) {
        use self::ControlPoint::*;

        match self {
            BezierPoint(x, y)           => (*x, *y),
            BezierControlPoint(x, y)    => (*x, *y)
        }
    }

    ///
    /// Returns true if this is a control point (vs a point on the curve)
    ///
    pub fn is_control_point(&self) -> bool {
        use self::ControlPoint::*;

        match self {
            BezierPoint(_, _)           => false,
            BezierControlPoint(_, _)    => true
        }
    }

    ///
    /// Applies a set of transformations to this control point
    ///
    pub fn apply_transformations<'a, TransformIter: IntoIterator<Item=&'a Transformation>>(&self, transform: TransformIter) -> ControlPoint {
        use self::ControlPoint::*;

        match self {
            BezierPoint(x, y)   => {
                let result = transform.into_iter().fold(Coord2(*x, *y), |coord, transform| transform.transform_point(&coord));
                BezierPoint(result.x(), result.y())
            }

            BezierControlPoint(x, y)   => {
                let result = transform.into_iter().fold(Coord2(*x, *y), |coord, transform| transform.transform_point(&coord));
                BezierControlPoint(result.x(), result.y())
            }
        }
    }
}
