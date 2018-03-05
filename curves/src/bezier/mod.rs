mod basis;
mod subdivide;
mod derivative;
mod tangent;
mod normal;
mod bounds;
mod deform;
mod fit;
mod offset;

pub use self::basis::*;
pub use self::subdivide::*;
pub use self::derivative::*;
pub use self::tangent::*;
pub use self::normal::*;
pub use self::bounds::*;
pub use self::deform::*;
pub use self::fit::*;
pub use self::offset::*;

use super::coordinate::*;

const LENGTH_SUBDIVISIONS: usize = 16;

///
/// Trait implemented by things representing a cubic bezier curve
/// 
pub trait BezierCurve: Clone+Sized {
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
    /// Reverses the direction of this curve
    /// 
    fn reverse(self) -> Self {
        let (cp1, cp2) = self.control_points();
        Self::from_points(self.end_point(), self.start_point(), cp2, cp1)
    }

    ///
    /// Given a value t from 0 to 1, returns a point on this curve
    /// 
    #[inline]
    fn point_at_pos(&self, t: f64) -> Self::Point {
        let control_points = self.control_points();
        de_casteljau4(t, self.start_point(), control_points.0, control_points.1, self.end_point())
    }

    ///
    /// Given a value t from 0 to 1, finds a point on this curve and subdivides it, returning the two resulting curves
    /// 
    #[inline]
    fn subdivide(&self, t: f64) -> (Self, Self) {
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
        let start       = self.start_point();
        let end         = self.end_point();
        let (cp1, cp2)  = self.control_points();

        bounding_box4(start, cp1, cp2, end)
    }

    ///
    /// Finds the t values where this curve has extremities
    /// 
    #[inline]
    fn find_extremities(&self) -> Vec<f64> {
        let start       = self.start_point();
        let end         = self.end_point();
        let (cp1, cp2)  = self.control_points();

        find_extremities(start, cp1, cp2, end)
    }

    ///
    /// Generates a curve by attempting to find a best fit against a set of points
    /// 
    #[inline]
    fn fit_from_points(points: &[Self::Point], max_error: f64) -> Option<Vec<Self>> {
        fit_curve(points, max_error)
    }

    ///
    /// Attempts to estimate the length of this curve
    /// 
    fn estimate_length(&self, max_t: f64) -> f64 {
        let mut last_pos = self.point_at_pos(0.0);
        let mut length   = 0.0;

        for t in 1..LENGTH_SUBDIVISIONS {
            let t           = (t as f64) / (LENGTH_SUBDIVISIONS as f64) * max_t;
            let next_pos    = self.point_at_pos(t);

            length += last_pos.distance_to(&next_pos);
            last_pos = next_pos;
        }

        length
    }
}

///
/// Represents a Bezier curve
/// 
#[derive(Clone, Copy, Debug)]
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

    #[inline]
    fn reverse(self) -> Curve {
        Curve {
            start_point:    self.end_point,
            end_point:      self.start_point,
            control_points: (self.control_points.1, self.control_points.0)
        }
    }
}
