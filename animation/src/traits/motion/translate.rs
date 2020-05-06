use super::transform::*;
use super::super::vector::*;
use super::super::time_path::*;

use smallvec::*;

use std::ops::Range;
use std::time::Duration;

///
/// Describes how an element is translated over time
///
#[derive(Clone, PartialEq, Debug)]
pub struct TranslateMotion {
    /// The point about which this transformation is taking place
    pub origin: (f32, f32),

    /// Curve describing where the origin moves to
    pub translate: TimeCurve
}

impl TranslateMotion {
    ///
    /// Creates a translate motion that just instantaneously moves something between two points
    ///
    pub fn move_to(when: Duration, from: (f32, f32), to: (f32, f32)) -> TranslateMotion {
        let to_point = TimePoint::new(to.0-from.0, to.1-from.1, when);

        TranslateMotion {
            origin:     from,
            translate:  TimeCurve::new(to_point, to_point)
        }
    }

    ///
    /// Sets the origin of this motion
    ///
    #[inline]
    pub fn set_origin(&mut self, new_origin: (f32, f32)) {
        self.origin = new_origin;
    }

    ///
    /// Sets the path of this motion
    ///
    #[inline]
    pub fn set_path(&mut self, new_path: TimeCurve) {
        self.translate = new_path;
    }
}

impl Default for TranslateMotion {
    ///
    /// Creates a defualt translate motion
    ///
    fn default() -> TranslateMotion {
        TranslateMotion {
            origin:     (0.0, 0.0),
            translate:  TimeCurve::new(TimePoint::new(0.0, 0.0, Duration::from_millis(0)), TimePoint::new(0.0, 0.0, Duration::from_millis(0)))
        }
    }
}

impl MotionTransform for TranslateMotion {
    fn range_millis(&self) -> Range<f32> {
        if self.translate.points.len() == 0 {
            0.0..0.0
        } else {
            let start   = self.translate.points[0].point.milliseconds();
            let end     = self.translate.points.last().unwrap().point.milliseconds();

            start..end
        }
    }

    ///
    /// Returns the transformations to apply for this motion at a particular point in time
    ///
    fn transformation(&self, when: Duration) -> SmallVec<[Transformation; 2]> {
        let time_millis = ((when.as_secs() as f32) * 1_000.0) + ((when.subsec_nanos() as f32) / 1_000_000.0);
        let origin      = self.origin;
        let position    = self.translate.point_at_time(time_millis);

        if let Some(position) = position {
            smallvec![Transformation::Translate((position.0-origin.0) as f64, (position.1-origin.1) as f64)]
        } else {
            smallvec![]
        }
    }
}
