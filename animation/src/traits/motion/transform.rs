use super::super::brush::*;

use std::ops::Range;
use std::time::Duration;

///
/// Trait implemented by motion objects that can help with transforming sets of points
/// 
pub trait MotionTransform {
    ///
    /// The range of times where this motion applies, in milliseconds
    /// 
    fn range_millis(&self) -> Range<f32>;

    ///
    /// Returns a transformed set of points at the specified time
    /// 
    fn transform_points<'a, Points: 'a+Iterator<Item=&'a BrushPoint>>(&self, time: Duration, points: Points) -> Box<'a+Iterator<Item=BrushPoint>>;
}
