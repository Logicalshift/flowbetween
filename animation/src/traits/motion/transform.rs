use super::super::raw_point::*;

use std::time::Duration;

///
/// Trait implemented by motion objects that can help with transforming sets of points
/// 
pub trait MotionTransform {
    ///
    /// Returns a transformed set of points at the specified time
    /// 
    fn transform_points<'a, Points: 'a+Iterator<Item=RawPoint>>(&self, time: Duration, points: Points) -> Box<'a+Iterator<Item=RawPoint>>;
}
