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

    ///
    /// Creates a cached animation for a set of region contents that don't change over time
    ///
    fn animate_cached<'a>(&'a self, region_contents: Arc<Vec<AnimationRegionContent>>) -> Box<dyn 'a+Fn(f64) -> Vec<AnimationPath>> {
        let cloned              = self.clone();
        let animation_function  = move |time| {
            cloned.animate(Arc::clone(&region_contents), time)
        };

        Box::new(animation_function)
    }
}
