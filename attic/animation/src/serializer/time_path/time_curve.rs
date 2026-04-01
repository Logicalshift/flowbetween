use super::super::source::*;
use super::super::target::*;
use super::super::super::traits::*;

use std::time::{Duration};

impl TimeCurve {
    ///
    /// Generates a serialized version of this time curve on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        data.write_usize(self.points.len());

        let mut last_point = TimePoint::new(0.0, 0.0, Duration::from_millis(0));
        for point in self.points.iter() {
            last_point = point.serialize_next(&last_point, data);
        }
    }
    
    ///
    /// Deserializes a time curve from the specified data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> TimeCurve {
        let num_points      = data.next_usize();

        let mut last_point  = TimePoint::new(0.0, 0.0, Duration::from_millis(0));
        let mut points      = vec![];
        for _ in 0..num_points {
            let (control_point, next_point) = TimeControlPoint::deserialize_next(&last_point, data);
            points.push(control_point);
            last_point = next_point;
        }

        TimeCurve { points }
    }
}
