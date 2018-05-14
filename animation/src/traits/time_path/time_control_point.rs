use super::time_point::*;

///
/// Represents a control point on a time curve
/// 
/// We always have a 'future' and 'past' control point, which means that the first point on a curve
/// has a superfluous 'past' point and the last has a superfluous 'future' point. This is so that
/// when considering an individual point we have control points for both how we arrive there and
/// how we leave.
/// 
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeControlPoint {
    /// The point that this represents
    pub point: TimePoint,

    /// The control point in the past
    pub past: TimePoint,

    /// The control point in the future
    pub future: TimePoint
}

impl TimeControlPoint {
    ///
    /// Creates a new time control point
    /// 
    pub fn new(past: TimePoint, point: TimePoint, future: TimePoint) -> TimeControlPoint {
        TimeControlPoint {
            past, point, future
        }
    }
}
