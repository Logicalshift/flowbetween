use super::time_point::*;
use super::time_control_point::*;

use curves::*;

use std::ops::{Mul,Add,Sub};

///
/// Represents a curve through time 
/// 
#[derive(Clone, PartialEq, Debug)]
pub struct TimeCurve {
    /// The points on this curves
    pub points: Vec<TimeControlPoint>
}

///
/// A section of a time curve
/// 
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TimeCurveSection {
    pub start: TimePoint,
    pub end: TimePoint,
    pub control_point1: TimePoint,
    pub control_point2: TimePoint
}

impl TimeCurve {
    ///
    /// Creates a new time curve from a line
    /// 
    pub fn new(start: TimePoint, end: TimePoint) -> TimeCurve {
        let start_point = TimeControlPoint::new(start, start, start + (end-start)*0.33333);
        let end_point   = TimeControlPoint::new(end - (end-start)*0.33333, end, end);

        TimeCurve { 
            points: vec![start_point, end_point]
        }
    }

    ///
    /// Returns the sections of this curve
    /// 
    pub fn as_sections(&self) -> Vec<TimeCurveSection> {
        let mut result = vec![];

        for index in 0..(self.points.len()-1) {
            result.push(TimeCurveSection {
                start:              self.points[index].point,
                end:                self.points[index+1].point,
                control_point1:     self.points[index].future,
                control_point2:     self.points[index+1].past
            })
        }

        result
    }
}

impl Mul<f64> for TimePoint {
    type Output=TimePoint;

    #[inline]
    fn mul(self, rhs: f64) -> TimePoint {
        let rhs = rhs as f32;
        TimePoint(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl Add<TimePoint> for TimePoint {
    type Output=TimePoint;

    #[inline]
    fn add(self, rhs: TimePoint) -> TimePoint {
        TimePoint(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl Sub<TimePoint> for TimePoint {
    type Output=TimePoint;

    #[inline]
    fn sub(self, rhs: TimePoint) -> TimePoint {
        TimePoint(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl Coordinate for TimePoint {
    ///
    /// Creates a new coordinate from the specified set of components
    /// 
    #[inline]
    fn from_components(components: &[f64]) -> TimePoint {
        TimePoint(components[0] as f32, components[1] as f32, components[2] as f32)
    }

    ///
    /// Returns the origin coordinate
    /// 
    #[inline]
    fn origin() -> TimePoint {
        TimePoint(0.0, 0.0, 0.0)
    }

    ///
    /// The number of components in this coordinate
    /// 
    #[inline]
    fn len() -> usize { 3 }

    ///
    /// Retrieves the component at the specified index
    /// 
    #[inline]
    fn get(&self, index: usize) -> f64 {
        match index {
            0 => self.0 as f64,
            1 => self.1 as f64,
            2 => self.2 as f64,
            _ => 0.0
        }
    }

    ///
    /// Returns a point made up of the biggest components of the two points
    /// 
    #[inline]
    fn from_biggest_components(p1: TimePoint, p2: TimePoint) -> TimePoint {
        TimePoint(p1.0.max(p2.0), p1.1.max(p2.1), p1.2.max(p2.2))
    }

    ///
    /// Returns a point made up of the smallest components of the two points
    /// 
    #[inline]
    fn from_smallest_components(p1: TimePoint, p2: TimePoint) -> TimePoint {
        TimePoint(p1.0.min(p2.0), p1.1.min(p2.1), p1.2.min(p2.2))
    }
}

impl BezierCurve for TimeCurveSection {
    type Point = TimePoint;

    ///
    /// Creates a new bezier curve of the same type from some points
    /// 
    #[inline]
    fn from_points(start: Self::Point, end: Self::Point, control_point1: Self::Point, control_point2: Self::Point) -> Self {
        TimeCurveSection {
            start, end, control_point1, control_point2
        }
    }

    ///
    /// The start point of this curve
    /// 
    #[inline]
    fn start_point(&self) -> Self::Point {
        self.start
    }

    ///
    /// The end point of this curve
    /// 
    #[inline]
    fn end_point(&self) -> Self::Point {
        self.end
    }

    ///
    /// The control points in this curve
    /// 
    #[inline]
    fn control_points(&self) -> (Self::Point, Self::Point) {
        (self.control_point1, self.control_point2)
    }
}