use std::time::{Duration};

use ::serde::*;

///
/// Structure representing when a frame occurs relative to the start of an animation
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FrameTime(i64);

impl FrameTime {
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
