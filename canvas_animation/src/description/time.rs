use super::space::*;

use serde::{Serialize, Deserialize};
use std::time::{Duration};

///
/// Represents a position at a particular time
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimePoint(pub Point2D, pub f64);

///
/// A point on a bezier path representing a motion through time
///
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeCurvePoint(pub TimePoint, pub TimePoint, pub TimePoint);

///
/// A bezier curve representing a motion through time
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
