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