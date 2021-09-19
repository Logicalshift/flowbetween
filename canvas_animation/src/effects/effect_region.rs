use crate::region::*;

use flo_curves::bezier::path::*;

use std::sync::*;
use std::time::{Duration};

///
/// Represents an animation effect that applies to a particular region
///
pub struct AnimationEffectRegion<TEffect: AnimationEffect> {
    /// The animation that will be performed within this region
    effect: TEffect,

    /// The region affected by the animation
    region: Vec<SimpleBezierPath>
}

///
/// Supplies a way to attach a region to an existing animation effect
///
pub trait AnimationEffectWithRegion : Sized+AnimationEffect {
    ///
    /// Adds an animation region to this effect
    ///
    fn with_region(self, region: Vec<SimpleBezierPath>) -> AnimationEffectRegion<Self>;
}

impl<TEffect: Sized+AnimationEffect> AnimationEffectWithRegion for TEffect {
    fn with_region(self, region: Vec<SimpleBezierPath>) -> AnimationEffectRegion<Self> {
        AnimationEffectRegion {
            effect: self,
            region: region
        }
    }
}

impl<TEffect: AnimationEffect> AnimationEffect for AnimationEffectRegion<TEffect> {
    fn duration(&self) -> Option<f64> { 
        self.effect.duration() 
    }

    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: Duration) -> Arc<AnimationRegionContent> { 
        self.effect.animate(region_contents, time) 
    }

    fn animate_cached(&self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn Fn(Duration) -> Arc<AnimationRegionContent>> { 
        self.effect.animate_cached(region_contents) 
    }
}

impl<TEffect: AnimationEffect> AnimationRegion for AnimationEffectRegion<TEffect> {
    fn region(&self, _time: Duration) -> Vec<SimpleBezierPath> {
        self.region.clone()
    }
}
