mod basis;
mod subdivide;
mod derivative;
mod tangent;
mod bounds;
mod deform;
mod fit;

pub use self::basis::*;
pub use self::subdivide::*;
pub use self::derivative::*;
pub use self::tangent::*;
pub use self::bounds::*;
pub use self::deform::*;
pub use self::fit::*;

use super::coordinate::*;

///
/// Trait implemented by things representing a cubic bezier curve
/// 
pub trait BezierCurve: Sized {
    type Point: Coordinate;

    ///
    /// Creates a new bezier curve of the same type from some points
    /// 
    fn from_points(start: Self::Point, end: Self::Point, control_point1: Self::Point, control_point2: Self::Point) -> Self;

    ///
    /// The start point of this curve
    /// 
    fn start_point(&self) -> Self::Point;

    ///
    /// The end point of this curve
    /// 
    fn end_point(&self) -> Self::Point;

    ///
    /// The control points in this curve
    /// 
    fn control_points(&self) -> (Self::Point, Self::Point);

    ///
    /// Given a value t from 0 to 1, returns a point on this curve
    /// 
    #[inline]
    fn point_at_pos(&self, t: f32) -> Self::Point {
        let control_points = self.control_points();
        de_casteljau4(t, self.start_point(), control_points.0, control_points.1, self.end_point())
    }

    ///
    /// Given a value t from 0 to 1, finds a point on this curve and subdivides it, returning the two resulting curves
    /// 
    #[inline]
    fn subdivide(&self, t: f32) -> (Self, Self) {
        let control_points              = self.control_points();
        let (first_curve, second_curve) = subdivide4(t, self.start_point(), control_points.0, control_points.1, self.end_point());

        (Self::from_points(first_curve.0, first_curve.3, first_curve.1, first_curve.2),
            Self::from_points(second_curve.0, second_curve.3, second_curve.1, second_curve.2))
    }

    ///
    /// Computes the bounds of this bezier curve
    /// 
    fn bounding_box(&self) -> (Self::Point, Self::Point) {
        // Fetch the various points and the derivative of this curve
        let start           = self.start_point();
        let end             = self.end_point();
        let control_points  = self.control_points();

        bounding_box4(start, control_points.0, control_points.1, end)
    }

    ///
    /// Generates a curve by attempting to find a best fit against a set of points
    /// 
    #[inline]
    fn fit_from_points(points: &[Self::Point], max_error: f32) -> Option<Vec<Self>> {
        fit_curve(points, max_error)
    }
}

///
/// Represents a Bezier curve
/// 
pub struct Curve {
    pub start_point:    Coord2,
    pub end_point:      Coord2,
    pub control_points: (Coord2, Coord2)
}

impl BezierCurve for Curve {
    type Point = Coord2;

    fn from_points(start: Coord2, end: Coord2, control_point1: Coord2, control_point2: Coord2) -> Curve {
        Curve {
            start_point:    start,
            end_point:      end,
            control_points: (control_point1, control_point2)
        }
    }

    #[inline]
    fn start_point(&self) -> Coord2 {
        self.start_point
    }

    #[inline]
    fn end_point(&self) -> Coord2 {
        self.end_point
    }

    #[inline]
    fn control_points(&self) -> (Coord2, Coord2) {
        self.control_points
    }
}
