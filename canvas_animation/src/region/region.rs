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
    /// Returns the definition of a region that this animation will affect from the static layer
    ///
    /// This will return the location of the region at a particular time so that drawing added after
    /// the initial keyframe will be 
    ///
    fn region(&self, region_number: usize, time: f64) -> Vec<SimpleBezierPath>;
}
