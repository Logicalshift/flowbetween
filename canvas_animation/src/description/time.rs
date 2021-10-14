use super::space::*;

use flo_curves::*;
use flo_curves::bezier::*;

use serde::{Serialize, Deserialize};

use std::ops::*;
use std::time::{Duration};

///
/// Represents a position at a particular time
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimePoint(pub Point2D, pub f64);

///
/// A point on a bezier path representing a motion through time
///
/// Format is two control points and an end point
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeCurvePoint(pub TimePoint, pub TimePoint, pub TimePoint);

///
/// A bezier curve representing a motion through time
///
/// Format is a start point and the points representing the 
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeCurve(pub TimePoint, pub TimeCurvePoint);

impl TimePoint {
    ///
    /// Converts a time point time to a Duration
    ///
    pub fn duration_from_f64(time: f64) -> Duration {
        let nanos = time * 1_000_000.0;
        let nanos = nanos as u64;
        Duration::from_nanos(nanos)
    }

    ///
    /// Converts a Duration to a time point time
    ///
    pub fn f64_from_duration(time: Duration) -> f64 {
        let nanos   = time.as_nanos();
        let millis  = (nanos as f64) / 1_000_000.0;

        millis
    }
}

impl Point2D {
    ///
    /// Adds a time to a point
    ///
    pub fn with_time(self, time: Duration) -> TimePoint {
        TimePoint(self, TimePoint::f64_from_duration(time))
    }
}

impl Add<TimePoint> for TimePoint {
    type Output=TimePoint;

    #[inline]
    fn add(self, rhs: TimePoint) -> TimePoint {
        TimePoint(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub<TimePoint> for TimePoint {
    type Output=TimePoint;

    #[inline]
    fn sub(self, rhs: TimePoint) -> TimePoint {
        TimePoint(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Mul<f64> for TimePoint {
    type Output=TimePoint;

    #[inline]
    fn mul(self, rhs: f64) -> TimePoint {
        TimePoint(self.0 * rhs, self.1 * rhs)
    }
}

impl Coordinate for TimePoint {
    #[inline]
    fn from_components(components: &[f64]) -> TimePoint {
        TimePoint(Point2D(components[0], components[1]), components[2])
    }

    #[inline]
    fn origin() -> TimePoint {
        TimePoint(Point2D::origin(), 0.0)
    }

    #[inline]
    fn len() -> usize { 3 }

    #[inline]
    fn get(&self, index: usize) -> f64 { 
        match index {
            0 => self.0.0,
            1 => self.0.1,
            2 => self.1,
            _ => panic!("TimePoint only has three components")
        }
    }

    fn from_biggest_components(p1: TimePoint, p2: TimePoint) -> TimePoint {
        TimePoint(Point2D::from_biggest_components(p1.0, p2.0), f64::from_biggest_components(p1.1, p2.1))
    }

    fn from_smallest_components(p1: TimePoint, p2: TimePoint) -> TimePoint {
        TimePoint(Point2D::from_smallest_components(p1.0, p2.0), f64::from_smallest_components(p1.1, p2.1))
    }

    #[inline]
    fn distance_to(&self, target: &TimePoint) -> f64 {
        let dist_x = target.0.0-self.0.0;
        let dist_y = target.0.1-self.0.1;
        let dist_t = target.1-self.1;

        f64::sqrt(dist_x*dist_x + dist_y*dist_y + dist_t*dist_t)
    }

    #[inline]
    fn dot(&self, target: &Self) -> f64 {
        self.0.0*target.0.0 + self.0.1*target.0.1 + self.1*target.1
    }
}

impl Into<Point2D> for TimePoint {
    fn into(self) -> Point2D {
        self.0
    }
}
