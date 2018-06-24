use super::geo::*;
use super::super::coordinate::*;

///
/// Trait implemented by things representing axis-aligned bounding boxes
/// 
pub trait BoundingBox : Geo {
    ///
    /// Returns a bounding box with the specified minimum and maximum coordinates
    /// 
    fn from_min_max(min: Self::Point, max: Self::Point) -> Self;

    ///
    /// Returns the minimum point of this bounding box
    ///
    fn min(&self) -> Self::Point;

    ///
    /// Returns the maximum point of this bounding box
    /// 
    fn max(&self) -> Self::Point;
}

///
/// Type representing a bounding box
/// 
/// (Unlike a normal point tuple this always represents its bounds in minimum/maximum order)
/// 
pub struct Bounds<Point: Coordinate>(Point, Point);

impl<Point: Coordinate> BoundingBox for (Point, Point) {
    #[inline]
    fn from_min_max(min: Self::Point, max: Self::Point) -> Self {
        (min, max)
    }

    #[inline]
    fn min(&self) -> Self::Point {
        Point::from_smallest_components(self.0, self.1)
    }

    #[inline]
    fn max(&self) -> Self::Point {
        Point::from_biggest_components(self.0, self.1)
    }
}

impl<Point: Coordinate> Geo for Bounds<Point> {
    type Point=Point;
}

impl<Point: Coordinate> BoundingBox for Bounds<Point> {
    #[inline]
    fn from_min_max(min: Self::Point, max: Self::Point) -> Self {
        Bounds(min, max)
    }

    #[inline]
    fn min(&self) -> Self::Point {
        self.0
    }

    #[inline]
    fn max(&self) -> Self::Point {
        self.1
    }
}