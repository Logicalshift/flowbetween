use super::super::vector::*;

use std::ops::Range;
use std::time::Duration;

use smallvec::*;

///
/// Trait implemented by motion objects that can help with transforming sets of points
///
pub trait MotionTransform {
    ///
    /// The range of times where this motion applies, in milliseconds
    ///
    fn range_millis(&self) -> Range<f32>;

    ///
    /// Returns the transformations to apply for this motion at a particular point in time
    ///
    fn transformation(&self, when: Duration) -> SmallVec<[Transformation; 2]>;
}
