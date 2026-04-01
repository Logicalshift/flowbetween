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
/// Format is a start point and the points representing the bezier curve
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeCurve(pub TimePoint, pub TimeCurvePoint);

///
/// A time transform point represents a transformation at a particular time
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeTransformPoint(pub TransformPoint, pub f64);

///
/// A time transform point is a point on a bezier path representing a transformation through time
///
/// Format is two control points and an end point
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeCurveTransformPoint(pub TimeTransformPoint, pub TimeTransformPoint, pub TimeTransformPoint);

///
/// A time transformation curve represents a transformation moving through time
///
/// Format sis a start point and the points representing the bezier curve
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeTransformCurve(pub TimeTransformPoint, pub TimeCurveTransformPoint);

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

impl Geo for TimeCurve {
    type Point = TimePoint;
}

impl BezierCurve for TimeCurve {
    ///
    /// The start point of this curve
    /// 
    fn start_point(&self) -> Self::Point { self.0 }

    ///
    /// The end point of this curve
    /// 
    fn end_point(&self) -> Self::Point { self.1.2 }

    ///
    /// The control points in this curve
    /// 
    fn control_points(&self) -> (Self::Point, Self::Point) {
        (self.1.0, self.1.1)
    }
}

impl TimeCurve {
    ///
    /// Finds the curve t value for a specified time
    ///
    pub fn t_for_time(&self, time: Duration) -> Option<f64> {
        let TimeCurve(TimePoint(_, t1), TimeCurvePoint(TimePoint(_, t2), TimePoint(_, t3), TimePoint(_, t4))) = self;

        let time        = TimePoint::f64_from_duration(time);
        let possible_t  = solve_basis_for_t(*t1, *t2, *t3, *t4, time);

        for t in possible_t {
            if t >= 0.0 && t <= 1.0 { return Some(t) }
        }

        None
    }

    ///
    /// Returns the point at a given t value
    ///
    pub fn point2d_for_t(&self, t: f64) -> Point2D {
        self.point_at_pos(t).into()
    }

    ///
    /// Returns the point at a given time
    ///
    pub fn point2d_for_time(&self, time: Duration) -> Point2D {
        if let Some(t) = self.t_for_time(time) {
            self.point_at_pos(t).into()
        } else {
            // Use the start or end point based on the mid-point of this curve
            let TimeCurve(TimePoint(p1, t1), TimeCurvePoint(TimePoint(_, _), TimePoint(_, _), TimePoint(p4, t4))) = self;
            let time = TimePoint::f64_from_duration(time);

            if time <= (*t1+*t4)*0.5 {
                *p1
            } else {
                *p4
            }
        }
    }
}

impl TimeTransformPoint {
    ///
    /// Converts a time point time to a Duration
    ///
    pub fn duration_from_f64(time: f64) -> Duration {
        TimePoint::duration_from_f64(time)
    }

    ///
    /// Converts a Duration to a time point time
    ///
    pub fn f64_from_duration(time: Duration) -> f64 {
        TimePoint::f64_from_duration(time)
    }
}

impl TransformPoint {
    ///
    /// Adds a time to a transform point
    ///
    pub fn with_time(self, time: Duration) -> TimeTransformPoint {
        TimeTransformPoint(self, TimeTransformPoint::f64_from_duration(time))
    }
}

impl Add<TimeTransformPoint> for TimeTransformPoint {
    type Output=TimeTransformPoint;

    #[inline]
    fn add(self, rhs: TimeTransformPoint) -> TimeTransformPoint {
        TimeTransformPoint(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub<TimeTransformPoint> for TimeTransformPoint {
    type Output=TimeTransformPoint;

    #[inline]
    fn sub(self, rhs: TimeTransformPoint) -> TimeTransformPoint {
        TimeTransformPoint(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Mul<f64> for TimeTransformPoint {
    type Output=TimeTransformPoint;

    #[inline]
    fn mul(self, rhs: f64) -> TimeTransformPoint {
        TimeTransformPoint(self.0 * rhs, self.1 * rhs)
    }
}

impl Coordinate for TimeTransformPoint {
    #[inline]
    fn from_components(components: &[f64]) -> TimeTransformPoint {
        TimeTransformPoint(TransformPoint::from_components(&components[0..=4]), components[5])
    }

    #[inline]
    fn origin() -> TimeTransformPoint {
        TimeTransformPoint(TransformPoint::origin(), 0.0)
    }

    #[inline]
    fn len() -> usize { 6 }

    #[inline]
    fn get(&self, index: usize) -> f64 { 
        match index {
            0 => self.0.get(0),
            1 => self.0.get(1),
            2 => self.0.get(2),
            3 => self.0.get(3),
            4 => self.0.get(4),
            5 => self.1,
            _ => panic!("TimeTransformPoint only has six components")
        }
    }

    fn from_biggest_components(p1: TimeTransformPoint, p2: TimeTransformPoint) -> TimeTransformPoint {
        TimeTransformPoint(TransformPoint::from_biggest_components(p1.0, p2.0), f64::from_biggest_components(p1.1, p2.1))
    }

    fn from_smallest_components(p1: TimeTransformPoint, p2: TimeTransformPoint) -> TimeTransformPoint {
        TimeTransformPoint(TransformPoint::from_smallest_components(p1.0, p2.0), f64::from_smallest_components(p1.1, p2.1))
    }
}

impl Into<TransformPoint> for TimeTransformPoint {
    fn into(self) -> TransformPoint {
        self.0
    }
}

impl TimeCurveTransformPoint {
    pub fn cp1(&self) -> TimeTransformPoint { self.0 }
    pub fn cp2(&self) -> TimeTransformPoint { self.1 }
    pub fn end_point(&self) -> TimeTransformPoint { self.2 }
}

impl Geo for TimeTransformCurve {
    type Point = TimeTransformPoint;
}

impl BezierCurve for TimeTransformCurve {
    ///
    /// The start point of this curve
    /// 
    fn start_point(&self) -> Self::Point { self.0 }

    ///
    /// The end point of this curve
    /// 
    fn end_point(&self) -> Self::Point { self.1.2 }

    ///
    /// The control points in this curve
    /// 
    fn control_points(&self) -> (Self::Point, Self::Point) {
        (self.1.0, self.1.1)
    }
}

impl BezierCurveFactory for TimeTransformCurve {
    ///
    /// Creates a new bezier curve of the same type from some points
    ///
    #[inline]
    fn from_points(start: Self::Point, (control_point1, control_point2): (Self::Point, Self::Point), end: Self::Point) -> TimeTransformCurve {
        TimeTransformCurve(start, TimeCurveTransformPoint(control_point1, control_point2, end))
    }
}

impl TimeTransformCurve {
    ///
    /// Finds the curve t value for a specified time
    ///
    pub fn t_for_time(&self, time: Duration) -> Option<f64> {
        let time = TimePoint::f64_from_duration(time);
        self.t_for_time_f64(time)
    }

    ///
    /// Finds the curve t value for a specified time
    ///
    pub fn t_for_time_f64(&self, time: f64) -> Option<f64> {
        let TimeTransformCurve(TimeTransformPoint(_, t1), TimeCurveTransformPoint(TimeTransformPoint(_, t2), TimeTransformPoint(_, t3), TimeTransformPoint(_, t4))) = self;

        let possible_t  = solve_basis_for_t(*t1, *t2, *t3, *t4, time);

        for t in possible_t {
            if t >= 0.0 && t <= 1.0 { return Some(t) }
        }

        None
    }

    ///
    /// Returns the point at a given t value
    ///
    pub fn transform_for_t(&self, t: f64) -> TransformPoint {
        self.point_at_pos(t).into()
    }

    ///
    /// Returns the point at a given time
    ///
    pub fn transform_for_time(&self, time: Duration) -> TransformPoint {
        let time = TimePoint::f64_from_duration(time);
        self.transform_for_time_f64(time)
    }

    ///
    /// Returns the point at a given time
    ///
    pub fn transform_for_time_f64(&self, time: f64) -> TransformPoint {
        if let Some(t) = self.t_for_time_f64(time) {
            self.point_at_pos(t).into()
        } else {
            // Use the start or end point based on the mid-point of this curve
            let TimeTransformCurve(TimeTransformPoint(p1, t1), TimeCurveTransformPoint(_, _, TimeTransformPoint(p4, t4))) = self;

            if time <= (*t1+*t4)*0.5 {
                *p1
            } else {
                *p4
            }
        }
    }
}
