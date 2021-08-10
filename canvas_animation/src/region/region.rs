use super::effect::*;

use flo_curves::bezier::path::*;

///
/// Represents a region of a vector layer that is animated
///
pub trait AnimationRegion : AnimationEffect {
    ///
    /// The number of seperate regions that this defines
    ///
    fn num_regions(&self) -> usize;

    ///
    /// Returns the definition of a sub-region that this animation will affect from the static layer
    ///
    /// This will return the location of the region at a particular time so that drawing added after
    /// the initial keyframe can be incorporated into the appropriate region
    ///
    fn region(&self, subregion_index: usize, time: f64) -> Vec<SimpleBezierPath>;
}

impl<T> AnimationRegion for Box<T>
where T: AnimationRegion {
    #[inline]
    fn num_regions(&self) -> usize { 
        (**self).num_regions() 
    }

    #[inline]
    fn region(&self, subregion_index: usize, time: f64) -> Vec<SimpleBezierPath> {
        (**self).region(subregion_index, time)
    }
}
