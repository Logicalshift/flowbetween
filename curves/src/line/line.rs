use super::coefficients::*;
use super::super::geo::*;
use super::super::coordinate::*;

///
/// Represents a straight line
/// 
pub trait Line : Geo {
    ///
    /// Creates a new line from points
    /// 
    fn from_points(p1: Self::Point, p2: Self::Point) -> Self;

    ///
    /// Returns the two points that mark the start and end of this line
    /// 
    fn points(&self) -> (Self::Point, Self::Point);

    ///
    /// Give a value 't' from 0 to 1, returns the point at that posiition along the line
    /// 
    fn point_at_pos(&self, t: f64) -> Self::Point {
        let (p1, p2)    = self.points();
        let delta       = p2-p1;

        p1 + delta*t
    }
}

///
/// Trait implemented by a 2D line
/// 
pub trait Line2D<Point2D: Coordinate+Coordinate2D> : Line<Point=Point2D> {
    ///
    /// Returns the coefficients (a, b, c) for this line, such that ax+by+c = 0 for
    /// any point on the line and also such that a^2 + b^2 = 1
    /// 
    #[inline]
    fn coefficients(&self) -> (f64, f64, f64) {
        line_coefficients_2d(self)
    }

    ///
    /// Returns the distance from a point to this line
    /// 
    #[inline]
    fn distance_to(&self, p: &Self::Point) -> f64 {
        let (a, b, c) = self.coefficients();

        a*p.x() + b*p.y() + c
    }
}

impl<Point: Coordinate+Clone> Geo for (Point, Point) {
    type Point = Point;
}

///
/// Simplest line is just a tuple of two points
/// 
impl<Point: Coordinate+Clone> Line for (Point, Point) {
    ///
    /// Creates a new line from points
    ///
    #[inline]
    fn from_points(p1: Self::Point, p2: Self::Point) -> Self {
        (p1, p2)
    }

    ///
    /// Returns the two points that mark the start and end of this line
    /// 
    #[inline]
    fn points(&self) -> (Self::Point, Self::Point) {
        self.clone()
    }
}

impl<Point: Coordinate2D+Coordinate+Clone> Line2D<Point> for (Point, Point) {
}
