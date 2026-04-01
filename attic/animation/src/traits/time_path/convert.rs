use std::time::Duration;

///
/// Converts a duration to a f64 representing the number of milliseconds it contains
///
#[inline]
pub fn to_millis(when: Duration) -> f64 {
    let secs    = when.as_secs() as f64;
    let nanos   = when.subsec_nanos() as f64;

    (secs * 1_000.0) + (nanos / 1_000_000.0)
}

///
/// Converts a f64 representing a number of milliseconds to a duration
///
#[inline]
pub fn to_duration(when: f64) -> Duration {
    if when < 0.0 {
        Duration::from_millis(0)
    } else {
        let secs    = (when/1000.0).floor();
        let nanos   = ((when - (secs*1000.0)) * 1_000_000.0).round();

        Duration::new(secs as u64, nanos as u32)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_convert_to_millis() {
        assert!((to_millis(Duration::from_millis(442))-442.0).abs() < 0.01);
    }

    #[test]
    fn can_convert_to_duration() {
        assert!(to_duration(442.0) == Duration::from_millis(442));
    }

    #[test]
    fn negative_millis_is_zero_duration() {
        assert!(to_duration(-442.0) == Duration::from_millis(0));
    }
}
