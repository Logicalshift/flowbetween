use std::time::Duration;

///
/// Represents a point in time
///
#[derive(Clone, Copy, Debug, PartialEq)]
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
    /// The time in milliseconds represented by this point
    ///
    #[inline]
    pub fn milliseconds(&self) -> f32 {
        self.2
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

    ///
    /// Returns the distance to another time point
    ///
    pub fn distance_to(&self, point: &TimePoint) -> f64 {
        let TimePoint(x1, y1, _)    = self;
        let TimePoint(x2, y2, _)    = point;
        let x1                      = *x1 as f64;
        let y1                      = *y1 as f64;
        let x2                      = *x2 as f64;
        let y2                      = *y2 as f64;

        f64::sqrt((x1-x2)*(x1-x2) + (y1-y2)*(y1-y2))
    }

    ///
    /// Returns true if this point is close to another
    ///
    pub fn is_close_to(&self, point: &TimePoint) -> bool {
        let TimePoint(x1, y1, t1)   = self;
        let TimePoint(x2, y2, t2)   = point;
        let x1                      = *x1 as f64;
        let y1                      = *y1 as f64;
        let t1                      = *t1 as f64;
        let x2                      = *x2 as f64;
        let y2                      = *y2 as f64;
        let t2                      = *t2 as f64;

        ((x1-x2)*(x1-x2) + (y1-y2)*(y1-y2) + (t1-t2)*(t1-t2)) < (0.1*0.1)
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
