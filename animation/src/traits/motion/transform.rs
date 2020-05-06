use super::super::path::*;
use super::super::brush::*;
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

    ///
    /// Returns a transformed set of points at the specified time
    ///
    fn transform_brush_points<'a, Points: 'a+Iterator<Item=&'a BrushPoint>>(&self, time: Duration, points: Points) -> Box<dyn 'a+Iterator<Item=BrushPoint>>;

    ///
    /// Returns a transformed set of points at the specified time
    ///
    fn transform_path_points<'a, Points: 'a+Iterator<Item=&'a PathPoint>>(&self, time: Duration, points: Points) -> Box<dyn 'a+Iterator<Item=PathPoint>>;

    ///
    /// For some points transformed by this motion, returns the original points
    ///
    /// This is particular useful when editing a transformed vector using the adjust tool: the tool
    /// needs to adjust the control points of the 'moved' element but adjust them properly for
    /// the underlying element.
    ///
    fn reverse_brush_points<'a, Points: 'a+Iterator<Item=&'a BrushPoint>>(&self, time: Duration, points: Points) -> Box<dyn 'a+Iterator<Item=BrushPoint>>;

    ///
    /// For some points transformed by this motion, returns the original points
    ///
    /// This is particularly useful when editing a transformed vector using the adjust tool: the tool
    /// needs to adjust the control points of the 'moved' element but adjust them properly for
    /// the underlying element.
    ///
    fn reverse_path_points<'a, Points: 'a+Iterator<Item=&'a PathPoint>>(&self, time: Duration, points: Points) -> Box<dyn 'a+Iterator<Item=PathPoint>>;
}
