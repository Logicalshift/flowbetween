use serde::{Serialize, Deserialize};

///
/// A point in 2D space
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point2D(pub f64, pub f64);

///
/// Two control points followed by an end point (a point on a bezier curve)
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BezierPoint(pub Point2D, pub Point2D, pub Point2D);

///
/// A path made up of bezier curves
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BezierPath(pub Point2D, pub Vec<BezierPoint>);

impl Point2D {
    #[inline] pub fn x(&self) -> f64 { self.0 }
    #[inline] pub fn y(&self) -> f64 { self.1 }
}
