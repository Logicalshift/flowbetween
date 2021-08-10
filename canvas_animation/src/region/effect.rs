use super::content::*;
use crate::animation_path::*;

use std::sync::*;

///
/// An animation effect describes how one or more sets of paths change over time
///
pub trait AnimationEffect : Clone+Send+Sync {
    ///
    /// Given the contents of the regions for this effect, calculates the path that should be rendered
    ///
    fn animate(&self, region_contents: Arc<Vec<AnimationRegionContent>>, time: f64) -> Vec<AnimationPath>;
}

impl<T> AnimationEffect for Box<T>
where T: AnimationEffect {
    #[inline]
    fn animate(&self, region_contents: Arc<Vec<AnimationRegionContent>>, time: f64) -> Vec<AnimationPath> {
        (**self).animate(region_contents, time)
    }
}
