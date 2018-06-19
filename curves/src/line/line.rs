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