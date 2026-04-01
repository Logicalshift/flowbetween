use std::time::{Duration};

///
/// The ways that time can be specified in FlowBetween
///
#[derive(Clone, Copy, PartialEq, Display, EnumString, EnumIter)]
pub enum TimeUnits {
    Frames,
    Seconds,
    Minutes
}

impl TimeUnits {
    ///
    /// Provides a description that can be used as a suffix for a value in these units
    ///
    pub fn description(&self) -> &'static str {
        use self::TimeUnits::*;

        match self {
            Frames  => "frames",
            Seconds => "seconds",
            Minutes => "minutes"
        }
    }

    ///
    /// Converts a Duration to a value in these time uniits
    ///
    pub fn from_duration(&self, time: Duration, frame_length: Duration) -> f64 {
        use self::TimeUnits::*;

        match self {
            Frames => {
                let nanos       = time.as_nanos() as f64;
                let frame_len   = frame_length.as_nanos() as f64;

                nanos / frame_len
            },

            Seconds => {
                let nanos       = time.as_nanos() as f64;
                nanos / 1_000_000_000.0
            }

            Minutes => {
                let nanos       = time.as_nanos() as f64;
                nanos / 1_000_000_000.0 / 60.0
            }
        }
    }

    ///
    /// Converts a value in these time units to a Duration
    ///
    pub fn to_duration(&self, time: f64, frame_length: Duration) -> Duration {
        use self::TimeUnits::*;

        match self {
            Frames => {
                let nanos = time * (frame_length.as_nanos() as f64);
                Duration::from_nanos(nanos as _)
            }

            Seconds => {
                let nanos = time * 1_000_000_000.0;
                Duration::from_nanos(nanos as _)
            }

            Minutes => {
                let nanos = time * 60.0 * 1_000_000_000.0;
                Duration::from_nanos(nanos as _)
            }
        }
    }
}
