use crate::path::*;
use super::content::*;

use std::sync::*;

///
/// An animation effect describes how one or more sets of paths change over time
///
pub trait AnimationEffect : Clone+Send+Sync {
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
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: f64) -> Vec<AnimationPath>;

    ///
    /// Given an input region that will remain fixed throughout the time period, returns a function that
    /// will animate it. This can be used to speed up operations when some pre-processing is required for
    /// the region contents, but is not always available as the region itself might be changing over time
    /// (eg, if many effects are combined)
    ///
    fn animate_cached<'a>(&'a self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn 'a+Fn(f64) -> Vec<AnimationPath>> {
        Box::new(move |time| {
            self.animate(Arc::clone(&region_contents), time)
        })
    }
}

impl<T> AnimationEffect for Box<T>
where T: AnimationEffect {
    #[inline]
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: f64) -> Vec<AnimationPath> {
        (**self).animate(region_contents, time)
    }

    #[inline]
    fn animate_cached<'a>(&'a self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn 'a+Fn(f64) -> Vec<AnimationPath>> {
        (**self).animate_cached(region_contents)
    }
}
