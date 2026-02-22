use std::time::{Duration};
use std::ops::*;

use ::serde::*;

///
/// Structure representing when a frame occurs relative to the start of an animation
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FrameTime(i64);

impl FrameTime {
    ///
    /// The earliest frame in the animation
    ///
    pub const ZERO: FrameTime = FrameTime(0);

    ///
    /// Create a FrameTime at a specific time in nanoseconds
    ///
    #[inline]
    pub fn from_nanos(nanoseconds: u64) -> Self {
        Self(nanoseconds as _)
    }

    ///
    /// Create a FrameTime at a time in seconds
    ///
    #[inline]
    pub fn from_seconds(seconds: f64) -> Self {
        let seconds = seconds.min(0.0);
        Self((seconds * 1_000_000_000.0) as _)
    }

    ///
    /// Returns the time in nanoseconds since the start of the animation for this frame
    ///
    #[inline]
    pub fn as_nanos(&self) -> i64 {
        self.0
    }
}

impl From<Duration> for FrameTime {
    #[inline]
    fn from(value: Duration) -> Self {
        Self(value.as_nanos() as _)
    }
}

impl Into<Duration> for FrameTime {
    #[inline]
    fn into(self) -> Duration {
        Duration::from_nanos(self.0 as _)
    }
}

impl Add for FrameTime {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
