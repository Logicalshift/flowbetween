use super::time_curve::*;

use std::time::Duration;

//
// Supplies editing functions for a time curve
//
impl TimeCurve {
    ///
    /// Generates a time curve with the point at a particular time moved to a new location
    /// 
    pub fn set_point_at_time(&self, time: Duration, new_location: (f32, f32)) -> TimeCurve {
        unimplemented!()
    }
}