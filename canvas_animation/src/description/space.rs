use serde::{Serialize, Deserialize};

use flo_curves::{Coordinate, Coordinate2D, Geo};
use flo_curves::bezier;

use std::ops::*;

// TODO: fix naming clash between BezierPath the structure and BezierPath the trait

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

impl Add<Point2D> for Point2D {
    type Output=Point2D;

    #[inline]
    fn add(self, rhs: Point2D) -> Point2D {
        Point2D(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub<Point2D> for Point2D {
    type Output=Point2D;

    #[inline]
    fn sub(self, rhs: Point2D) -> Point2D {
        Point2D(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Mul<f64> for Point2D {
    type Output=Point2D;

    #[inline]
    fn mul(self, rhs: f64) -> Point2D {
        Point2D(self.0 * rhs, self.1 * rhs)
    }
}

impl Coordinate for Point2D {
    #[inline]
    fn from_components(components: &[f64]) -> Point2D {
        Point2D(components[0], components[1])
    }

    #[inline]
    fn origin() -> Point2D {
        Point2D(0.0, 0.0)
    }

    #[inline]
    fn len() -> usize { 2 }

    #[inline]
    fn get(&self, index: usize) -> f64 { 
        match index {
            0 => self.0,
            1 => self.1,
            _ => panic!("Point2D only has two components")
        }
    }

    fn from_biggest_components(p1: Point2D, p2: Point2D) -> Point2D {
        Point2D(f64::from_biggest_components(p1.0, p2.0), f64::from_biggest_components(p1.1, p2.1))
    }

    fn from_smallest_components(p1: Point2D, p2: Point2D) -> Point2D {
        Point2D(f64::from_smallest_components(p1.0, p2.0), f64::from_smallest_components(p1.1, p2.1))
    }

    #[inline]
    fn distance_to(&self, target: &Point2D) -> f64 {
        let dist_x = target.0-self.0;
        let dist_y = target.1-self.1;

        f64::sqrt(dist_x*dist_x + dist_y*dist_y)
    }

    #[inline]
    fn dot(&self, target: &Self) -> f64 {
        self.0*target.0 + self.1*target.1
    }
}

impl Coordinate2D for Point2D {
    #[inline]
    fn x(&self) -> f64 {
        self.0
    }

    #[inline]
    fn y(&self) -> f64 {
        self.1
    }
}

impl Geo for BezierPath {
    type Point = Point2D;
}

impl bezier::path::BezierPath for BezierPath {
    type PointIter = Box<dyn Iterator<Item=(Self::Point, Self::Point, Self::Point)>>;

    fn start_point(&self) -> Self::Point {
        self.0
    }

    fn points(&self) -> Self::PointIter {
        Box::new(self.1.clone()
            .into_iter()
            .map(|BezierPoint(cp1, cp2, ep)| (cp1, cp2, ep)))
    }
}

impl bezier::path::BezierPathFactory for BezierPath {
    fn from_points<FromIter: IntoIterator<Item=(Self::Point, Self::Point, Self::Point)>>(start_point: Self::Point, points: FromIter) -> Self {
        BezierPath(start_point, points.into_iter().map(|(cp1, cp2, ep)| BezierPoint(cp1, cp2, ep)).collect())
    }
}
