use std::time::Duration;

///
/// Represents a point in time
/// 
pub struct TimePoint(pub f32, pub f32, pub f32);

impl TimePoint {
    ///
    /// Creates a point with a location at a particular time
    /// 
    pub fn new(x: f32, y: f32, time: Duration) -> TimePoint {
        let secs    = time.as_secs() as f32;
        let nanos   = time.subsec_nanos() as f32;

        let time    = (secs * 1_000.0) + (nanos / 1_000_000.0);

        TimePoint(x, y, time)
    }

    ///
    /// Retrieves the x and y coordinates of this point
    /// 
    pub fn coords(&self) -> (f32, f32) {
        let TimePoint(x, y, _) = self;

        (*x, *y)
    }

    ///
    /// Retrieves the time for this point as a duration
    /// 
    pub fn time(&self) -> Duration {
        let TimePoint(_, _, millis) = self;

        let secs    = (millis / 1000.0).floor();
        let nanos   = (((millis / 1000.0)-secs).abs() * 1_000_000.0).round() * 1_000.0;

        if secs < 0.0 {
            // Negative durations are not supported
            Duration::from_millis(0)
        } else {
            Duration::new(secs as u64, nanos as u32)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn can_round_trip_duration_mills() {
        assert!(TimePoint::new(0.0, 0.0, Duration::from_millis(1234)).time() == Duration::from_millis(1234));
    }
}
