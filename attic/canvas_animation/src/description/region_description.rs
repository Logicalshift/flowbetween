use super::space::*;
use super::effect_description::*;

use serde::{Serialize, Deserialize};

///
/// Describes a region and the effect that applies to it
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegionDescription(pub Vec<BezierPath>, pub EffectDescription);

impl RegionDescription {
    ///
    /// The path that this animation region affects
    ///
    pub fn region(&self) -> &Vec<BezierPath> {
        &self.0
    }

    ///
    /// The effect that's applied to the animation region
    ///
    pub fn effect(&self) -> &EffectDescription {
        &self.1
    }

    ///
    /// The effect that's applied to the animation region
    ///
    pub fn effect_mut(&mut self) -> &mut EffectDescription {
        &mut self.1
    }
}
