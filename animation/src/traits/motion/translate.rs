use super::transform::*;
use super::super::brush::*;
use super::super::time_path::*;

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

    fn transform_points<'a, Points: 'a+Iterator<Item=&'a BrushPoint>>(&self, time: Duration, points: Points) -> Box<'a+Iterator<Item=BrushPoint>> {
        let time_millis = ((time.as_secs() as f32) * 1_000.0) + ((time.subsec_nanos() as f32) / 1_000_000.0);
        let origin      = self.origin;
        let position    = self.translate.point_at_time(time_millis);

        if let Some(position) = position {
            // Points mapped to a new position
            let offset  = (position.0 - origin.0, position.1 - origin.1);

            Box::new(points.map(move |point| {
                BrushPoint {
                    position:   (point.position.0 + offset.0, point.position.1 + offset.1),
                    cp1:        (point.cp1.0 + offset.0, point.cp1.1 + offset.1),
                    cp2:        (point.cp2.0 + offset.0, point.cp2.1 + offset.1),
                    width:      point.width,
                }
            }))
        } else {
            // Points unchanged if we can't find a time
            Box::new(points.cloned())
        }
    }
}
