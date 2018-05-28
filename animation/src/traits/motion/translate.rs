use super::transform::*;
use super::super::time_path::*;
use super::super::raw_point::*;

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

    fn transform_points<'a, Points: 'a+Iterator<Item=RawPoint>>(&self, time: Duration, points: Points) -> Box<'a+Iterator<Item=RawPoint>> {
        let time_millis = ((time.as_secs() as f32) * 1_000.0) + ((time.subsec_nanos() as f32) / 1_000_000.0);
        let origin      = self.origin;
        let position    = self.translate.point_at_time(time_millis);

        if let Some(position) = position {
            // Points mapped to a new position
            let offset  = (position.0 - origin.0, position.1 - origin.1);

            Box::new(points.map(move |point| {
                RawPoint {
                    position:   (point.position.0 + offset.0, point.position.1 + offset.1),
                    pressure:   point.pressure,
                    tilt:       point.tilt
                }
            }))
        } else {
            // Points unchanged if we can't find a time
            Box::new(points)
        }
    }
}
