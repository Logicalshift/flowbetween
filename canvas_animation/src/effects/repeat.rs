use crate::region::*;

use std::sync::*;
use std::time::{Duration};

///
/// Animation effect that repeats another effect after a set duration (starting the time from 0 again)
///
pub struct RepeatEffect<TEffect: AnimationEffect> {
    /// The effect that will be repeated
    effect: TEffect,

    /// The time that the effect can run for before being repeated
    repeat_time: Duration
}

impl<TEffect: AnimationEffect> RepeatEffect<TEffect> {
    ///
    /// Creates a new repeating animation effect
    ///
    pub fn repeat_effect(effect: TEffect, repeat_time: Duration) -> RepeatEffect<TEffect> {
        RepeatEffect {
            effect:         effect,
            repeat_time:    repeat_time
        }
    }

    ///
    /// Returns the time to use for the internal effect given the overall animation time
    ///
    fn time_for_time(repeat_time: Duration, time: Duration) -> Duration {
        Duration::from_nanos((time.as_nanos() % repeat_time.as_nanos()) as _)
    }
}

impl<TEffect: AnimationEffect> AnimationEffect for RepeatEffect<TEffect> {
    ///
    /// Given the contents of the regions for this effect, calculates the path that should be rendered
    ///
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: Duration) -> Arc<AnimationRegionContent> {
        let time = Self::time_for_time(self.repeat_time, time);

        self.effect.animate(region_contents, time)
    }

    ///
    /// Given an input region that will remain fixed throughout the time period, returns a function that
    /// will animate it. This can be used to speed up operations when some pre-processing is required for
    /// the region contents, but is not always available as the region itself might be changing over time
    /// (eg, if many effects are combined)
    ///
    fn animate_cached(&self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn Fn(Duration) -> Arc<AnimationRegionContent>> {
        let cached_effect   = self.effect.animate_cached(region_contents);
        let repeat_time     = self.repeat_time;

        Box::new(move |time| {
            let time = Self::time_for_time(repeat_time, time);
            cached_effect(time)
        })
    }
}
