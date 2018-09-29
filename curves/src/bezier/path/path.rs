use super::bounds::*;
use super::to_curves::*;
use super::super::curve::*;
use super::super::super::geo::*;
use super::super::super::coordinate::*;

use std::vec;

///
/// Trait representing a path made out of bezier sections
/// 
pub trait BezierPath : Geo+Clone+Sized {
    /// Type of an iterator over the points in this curve. This tuple contains the points ordered as a hull: ie, two control points followed by a point on the curve
    type PointIter: Iterator<Item=(Self::Point, Self::Point, Self::Point)>;

    ///
    /// Retrieves the initial point of this path
    /// 
    fn start_point(&self) -> Self::Point;

    ///
    /// Retrieves an iterator over the points in this path
    /// 
    fn points(&self) -> Self::PointIter;

    ///
    /// Finds the bounds of this path
    /// 
    #[inline]
    fn bounding_box<Bounds: BoundingBox<Point=Self::Point>>(&self) -> Bounds {
        path_bounding_box(self)
    }

    ///
    /// Finds a loose bounding box for this path (more quickly than bounding_box)
    /// 
    /// This will contain the path but might not be tightly aligned to the curves
    ///
    fn fast_bounding_box<Bounds: BoundingBox<Point=Self::Point>>(&self) -> Bounds {
        path_fast_bounding_box(self)
    }

    ///
    /// Changes this path into a set of bezier curves
    /// 
    #[inline]
    fn to_curves<Curve: BezierCurveFactory<Point=Self::Point>>(&self) -> Vec<Curve> {
        path_to_curves(self).collect()
    }
}

///
/// Trait implemented by types that can construct new bezier paths
///
pub trait BezierPathFactory : BezierPath {
    ///
    /// Creates a new instance of this path from a set of points
    /// 
    fn from_points<FromIter: IntoIterator<Item=(Self::Point, Self::Point, Self::Point)>>(start_point: Self::Point, points: FromIter) -> Self;

    ///
    /// Creates a new instance of this path from the points in another path
    ///
    fn from_path<FromPath: BezierPath<Point=Self::Point>>(path: &FromPath) -> Self {
        Self::from_points(path.start_point(), path.points())
    }
}

impl<Point: Clone+Coordinate> Geo for (Point, Vec<(Point, Point, Point)>) {
    type Point = Point;
}

///
/// The type (start_point, Vec<(Point, Point, Point)>) is the simplest bezier path type
/// 
impl<Point: Clone+Coordinate> BezierPath for (Point, Vec<(Point, Point, Point)>) {
    type PointIter  = vec::IntoIter<(Point, Point, Point)>;

    ///
    /// Retrieves the initial point of this path
    /// 
    fn start_point(&self) -> Self::Point {
        self.0.clone()
    }

    ///
    /// Retrieves an iterator over the points in this path
    /// 
    fn points(&self) -> Self::PointIter {
        self.1.clone().into_iter()
    }
}

impl<Point: Clone+Coordinate> BezierPathFactory for (Point, Vec<(Point, Point, Point)>) {
    ///
    /// Creates a new instance of this path from a set of points
    /// 
    fn from_points<FromIter: IntoIterator<Item=(Self::Point, Self::Point, Self::Point)>>(start_point: Self::Point, points: FromIter) -> Self {
        (start_point, points.into_iter().collect())
    }
}

/// Basic Bezier path type
pub type SimpleBezierPath = (Coord2, Vec<(Coord2, Coord2, Coord2)>);
