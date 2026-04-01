// FlowBetween, a tool for creating vector animations
// Copyright (C) 2026 Andrew Hunter
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
