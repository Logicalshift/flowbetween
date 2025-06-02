use flo_curves::geo::*;
use flo_curves::bezier::path::*;

use serde::*;
use std::vec;
use std::ops::{Add, Sub, Mul, Div};

///
/// Coordinate in the user interface
///
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub struct UiPoint(pub f64, pub f64);

///
/// A bezier path in the user interface
///
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct UiPath {
    points: (UiPoint, Vec<(UiPoint, UiPoint, UiPoint)>),
}

impl UiPoint {
    ///
    /// Returns true if this point is within the specified bounding box
    ///
    #[inline]
    pub fn in_bounds(&self, bounds: &impl BoundingBox<Point=UiPoint>) -> bool {
        let min = bounds.min();
        let max = bounds.max();

        min.x() <= self.0 
            && min.y() <= self.1 
            && max.x() >= self.0 
            && max.y() >= self.1
    }
}

impl Coordinate2D for UiPoint {
    ///
    /// X component of this coordinate
    /// 
    #[inline]
    fn x(&self) -> f64 {
        self.0
    }

    ///
    /// Y component of this coordinate
    /// 
    #[inline]
    fn y(&self) -> f64 {
        self.1
    }
}

impl Add<UiPoint> for UiPoint {
    type Output=UiPoint;

    #[inline]
    fn add(self, rhs: UiPoint) -> UiPoint {
        UiPoint(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub<UiPoint> for UiPoint {
    type Output=UiPoint;

    #[inline]
    fn sub(self, rhs: UiPoint) -> UiPoint {
        UiPoint(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Mul<f64> for UiPoint {
    type Output=UiPoint;

    #[inline]
    fn mul(self, rhs: f64) -> UiPoint {
        UiPoint(self.0 * rhs, self.1 * rhs)
    }
}

impl From<(f64, f64)> for UiPoint {
    fn from((x, y): (f64, f64)) -> UiPoint {
        UiPoint(x, y)
    }
}

impl Into<(f64, f64)> for UiPoint {
    fn into(self) -> (f64, f64) {
        (self.0, self.1)
    }
}

impl From<(f32, f32)> for UiPoint {
    fn from((x, y): (f32, f32)) -> UiPoint {
        UiPoint(x as _, y as _)
    }
}

impl Into<(f32, f32)> for UiPoint {
    fn into(self) -> (f32, f32) {
        (self.0 as _, self.1 as _)
    }
}

impl Coordinate for UiPoint {
    #[inline]
    fn from_components(components: &[f64]) -> UiPoint {
        UiPoint(components[0], components[1])
    }

    #[inline]
    fn origin() -> UiPoint {
        UiPoint(0.0, 0.0)
    }

    #[inline]
    fn len() -> usize { 2 }

    #[inline]
    fn get(&self, index: usize) -> f64 { 
        match index {
            0 => self.0,
            1 => self.1,
            _ => panic!("UiPoint only has two components")
        }
    }

    fn from_biggest_components(p1: UiPoint, p2: UiPoint) -> UiPoint {
        UiPoint(f64::from_biggest_components(p1.0, p2.0), f64::from_biggest_components(p1.1, p2.1))
    }

    fn from_smallest_components(p1: UiPoint, p2: UiPoint) -> UiPoint {
        UiPoint(f64::from_smallest_components(p1.0, p2.0), f64::from_smallest_components(p1.1, p2.1))
    }

    #[inline]
    fn distance_to(&self, target: &UiPoint) -> f64 {
        let dist_x = target.0-self.0;
        let dist_y = target.1-self.1;

        f64::sqrt(dist_x*dist_x + dist_y*dist_y)
    }

    #[inline]
    fn dot(&self, target: &Self) -> f64 {
        self.0*target.0 + self.1*target.1
    }
}

impl From<Coord2> for UiPoint {
    fn from(coord2: Coord2) -> UiPoint {
        UiPoint(coord2.x(), coord2.y())
    }
}

impl<'a> From<&'a Coord2> for UiPoint {
    fn from(coord2: &'a Coord2) -> UiPoint {
        UiPoint(coord2.x(), coord2.y())
    }
}

impl Geo for UiPath {
    type Point = UiPoint;
}

impl BezierPath for UiPath {
    type PointIter  = vec::IntoIter<(UiPoint, UiPoint, UiPoint)>;

    ///
    /// Retrieves the initial point of this path
    /// 
    fn start_point(&self) -> UiPoint {
        self.points.0
    }

    ///
    /// Retrieves an iterator over the points in this path
    /// 
    fn points(&self) -> Self::PointIter {
        self.points.1.clone().into_iter()
    }
}

impl BezierPathFactory for UiPath {
    ///
    /// Creates a new instance of this path from a set of points
    /// 
    fn from_points<FromIter: IntoIterator<Item=(UiPoint, UiPoint, UiPoint)>>(start_point: Self::Point, points: FromIter) -> Self {
        UiPath {
            points: (start_point, points.into_iter().collect())
        }
    }
}

