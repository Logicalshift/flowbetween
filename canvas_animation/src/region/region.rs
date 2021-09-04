use crate::path::*;
use super::effect::*;
use super::content::*;

use flo_curves::bezier::path::*;

use std::sync::*;
use std::time::{Duration};

///
/// Represents a region of a vector layer that is animated
///
pub trait AnimationRegion : AnimationEffect {
    ///
    /// Returns the definition of a sub-region that this animation will affect from the static layer
    ///
    /// This will return the location of the region at a particular time so that drawing added after
    /// the initial keyframe can be incorporated into the appropriate region
    ///
    fn region(&self, time: Duration) -> Vec<SimpleBezierPath>;
}

impl<T> AnimationRegion for Box<T>
where T: AnimationRegion {
    #[inline]
    fn region(&self, time: Duration) -> Vec<SimpleBezierPath> {
        (**self).region(time)
    }
}

impl AnimationEffect for Arc<dyn AnimationRegion> {
    #[inline]
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: Duration) -> Vec<AnimationPath> {
        (**self).animate(region_contents, time)
    }

    #[inline]
    fn animate_cached<'a>(&'a self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn 'a+Fn(Duration) -> Vec<AnimationPath>> {
        (**self).animate_cached(region_contents)
    }
}

impl AnimationRegion for Arc<dyn AnimationRegion> {
    #[inline]
    fn region(&self, time: Duration) -> Vec<SimpleBezierPath> {
        (**self).region(time)
    }
}

impl<T> AnimationRegion for Arc<T>
where T: AnimationRegion {
    #[inline]
    fn region(&self, time: Duration) -> Vec<SimpleBezierPath> {
        (**self).region(time)
    }
}
