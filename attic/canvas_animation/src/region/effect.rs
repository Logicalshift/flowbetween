use super::content::*;

use std::sync::*;
use std::time::{Duration};

// TODO: add a way to specify a 2D transformation at a time for simpler effects

///
/// An animation effect describes how one or more sets of paths change over time
///
pub trait AnimationEffect : Send+Sync {
    ///
    /// Returns the duration of this effect (or None if this effect will animate forever)
    ///
    /// If the effect is passed a time that's after where the 'duration' has completed it should always generate the same result
    ///
    fn duration(&self) -> Option<f64> {
        None
    }

    ///
    /// Given the contents of the regions for this effect, calculates the path that should be rendered
    ///
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: Duration) -> Arc<AnimationRegionContent>;

    ///
    /// Given an input region that will remain fixed throughout the time period, returns a function that
    /// will animate it. This can be used to speed up operations when some pre-processing is required for
    /// the region contents, but is not always available as the region itself might be changing over time
    /// (eg, if many effects are combined)
    ///
    fn animate_cached(&self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn Send+Fn(Duration) -> Arc<AnimationRegionContent>>;
}

impl<T> AnimationEffect for Box<T>
where T: AnimationEffect {
    #[inline]
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: Duration) -> Arc<AnimationRegionContent> {
        (**self).animate(region_contents, time)
    }

    #[inline]
    fn animate_cached(&self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn Send+Fn(Duration) -> Arc<AnimationRegionContent>> {
        (**self).animate_cached(region_contents)
    }
}

impl AnimationEffect for Box<dyn AnimationEffect> {
    #[inline]
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: Duration) -> Arc<AnimationRegionContent> {
        (**self).animate(region_contents, time)
    }

    #[inline]
    fn animate_cached(&self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn Send+Fn(Duration) -> Arc<AnimationRegionContent>> {
        (**self).animate_cached(region_contents)
    }
}

impl<T> AnimationEffect for Arc<T>
where T: AnimationEffect {
    #[inline]
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: Duration) -> Arc<AnimationRegionContent> {
        (**self).animate(region_contents, time)
    }

    #[inline]
    fn animate_cached(&self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn Send+Fn(Duration) -> Arc<AnimationRegionContent>> {
        (**self).animate_cached(region_contents)
    }
}
