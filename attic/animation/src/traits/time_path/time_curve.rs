use super::time_point::*;
use super::time_control_point::*;

use flo_curves::*;

use std::ops::{Mul,Add,Sub};

/// Number of milliseconds precision to use for times
const DELTA: f32 = 0.1;

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
    pub control_point1: TimePoint,
    pub control_point2: TimePoint,
    pub end: TimePoint
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

    ///
    /// Finds the point within this curve at the specified time
    ///
    pub fn point_at_time(&self, milliseconds: f32) -> Option<TimePoint> {
        self.as_sections()
            .into_iter()
            .filter(|section| {
                // Filter to sections possibly containing this time
                let (min, max) = section.bounding_box();

                min.milliseconds() <= milliseconds && max.milliseconds() >= milliseconds
            })
            .map(|section| section.point_at_time(milliseconds))
            .filter(|time| time.is_some())
            .nth(0)
            .unwrap_or(None)
    }

    ///
    /// Returns true if this curve is a close match to another (mainly useful for testing)
    ///
    pub fn is_close_to(&self, other_curve: &TimeCurve) -> bool {
        if other_curve.points.len() != self.points.len() {
            // Both curves must have the same number of points
            false
        } else {
            // Each control point must be near the corresponding point in the other curve
            self.points.iter()
                .zip(other_curve.points.iter())
                .all(|(p1, p2)| {
                    (p1.past).is_close_to(&p2.past) &&
                    (p1.point).is_close_to(&p2.point) &&
                    (p1.future).is_close_to(&p2.future)
                })
        }
    }
}

impl TimeCurveSection {
    ///
    /// Solves for the point on this curve at the specified time (if it exists)
    ///
    pub fn point_at_time(&self, milliseconds: f32) -> Option<TimePoint> {
        let t_points = self.search_with_bounds(DELTA as f64, |min, max| min.milliseconds() <= milliseconds && max.milliseconds() >= milliseconds);

        t_points.into_iter()
            .nth(0)
            .map(|point_t| self.point_at_pos(point_t))
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

impl Geo for TimeCurveSection {
    type Point = TimePoint;
}

impl BezierCurveFactory for TimeCurveSection {
    ///
    /// Creates a new bezier curve of the same type from some points
    ///
    #[inline]
    fn from_points(start: Self::Point, (control_point1, control_point2): (Self::Point, Self::Point), end: Self::Point) -> Self {
        TimeCurveSection {
            start, control_point1, control_point2, end
        }
    }
}

impl BezierCurve for TimeCurveSection {
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

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    pub fn can_find_initial_point() {
        let time_curve          = TimeCurve::new(TimePoint::new(20.0, 30.0, Duration::from_millis(0)), TimePoint::new(130.0, 110.0, Duration::from_millis(1000)));

        let point_at_start      = time_curve.point_at_time(0.0).unwrap();
        let distance_from_start = point_at_start.distance_to(&TimePoint::new(20.0, 30.0, Duration::from_millis(0)));

        assert!(distance_from_start <= DELTA as f64);
    }

    #[test]
    pub fn can_find_final_point() {
        let time_curve          = TimeCurve::new(TimePoint::new(20.0, 30.0, Duration::from_millis(0)), TimePoint::new(130.0, 110.0, Duration::from_millis(1000)));

        let point_at_end        = time_curve.point_at_time(1000.0).unwrap();
        let distance_from_end   = point_at_end.distance_to(&TimePoint::new(130.0, 110.0, Duration::from_millis(1000)));

        assert!(distance_from_end <= DELTA as f64);
    }

    #[test]
    pub fn can_find_mid_point() {
        let time_curve          = TimeCurve::new(TimePoint::new(20.0, 30.0, Duration::from_millis(0)), TimePoint::new(130.0, 110.0, Duration::from_millis(1000)));

        let point_at_mid        = time_curve.point_at_time(500.0).unwrap();
        let distance_from_mid   = point_at_mid.distance_to(&TimePoint::new(75.0, 70.0, Duration::from_millis(500)));

        assert!(distance_from_mid <= DELTA as f64);
    }

    #[test]
    pub fn default_time_line_is_linear() {
        let start_point         = TimePoint::new(20.0, 30.0, Duration::from_millis(0));
        let end_point           = TimePoint::new(130.0, 110.0, Duration::from_millis(1000));
        let time_curve          = TimeCurve::new(start_point, end_point);

        for point in 0..=10 {
            let point   = point as f32;
            let ratio   = point / 10.0;
            let time    = 1000.0 * ratio;

            let expected_point  = ((end_point-start_point) * (ratio as f64)) + start_point;
            let actual_point    = time_curve.point_at_time(time).unwrap();

            let distance        = expected_point.distance_to(&actual_point);
            assert!(distance < 1.0);
        }
    }

    #[test]
    pub fn default_time_line_is_linear_2() {
        let start_point         = TimePoint::new(20.0, 30.0, Duration::from_millis(0));
        let end_point           = TimePoint::new(130.0, 110.0, Duration::from_millis(1000));
        let time_curve          = TimeCurve::new(start_point, end_point).as_sections()[0];

        for point in 0..=10 {
            let point   = point as f32;
            let ratio   = point / 10.0;

            let expected_point  = ((end_point-start_point) * (ratio as f64)) + start_point;
            let actual_point    = time_curve.point_at_pos(ratio as f64);

            let distance        = expected_point.distance_to(&actual_point);
            assert!(distance < 1.0);
        }
    }

    #[test]
    pub fn points_outside_curve_are_not_found() {
        let time_curve          = TimeCurve::new(TimePoint::new(20.0, 30.0, Duration::from_millis(0)), TimePoint::new(130.0, 110.0, Duration::from_millis(1000)));

        assert!(time_curve.point_at_time(-100.0) == None);
        assert!(time_curve.point_at_time(1001.0) == None);
    }
}
