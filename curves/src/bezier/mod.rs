use super::coordinate::*;

///
/// Trait implemented by things representing a cubic bezier curve
/// 
pub trait BezierCurve<Point: Coordinate> {
    ///
    /// Creates a new bezier curve of the same type from some points
    /// 
    fn from_points(start: Point, end: Point, control_point1: Point, control_point2: Point) -> Self;

    ///
    /// The start point of this curve
    /// 
    fn start_point(&self) -> Point;

    ///
    /// The end point of this curve
    /// 
    fn end_point(&self) -> Point;

    ///
    /// The control points in this curve
    /// 
    fn control_points(&self) -> (Point, Point);
}

///
/// Represents a Bezier curve
/// 
pub struct Curve {
    pub start_point:    Coord2,
    pub end_point:      Coord2,
    pub control_points: (Coord2, Coord2)
}

impl BezierCurve<Coord2> for Curve {
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

mod basis;
mod subdivide;

pub use self::basis::*;
pub use self::subdivide::*;
