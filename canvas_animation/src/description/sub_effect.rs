use super::effect_description::*;

use smallvec::*;
use serde::{Serialize, Deserialize};

///
/// A sub-effect represents a single part of an effect description
///
/// These can be edited individually in the UI. A sub-effect type always corresponds to a
/// specific type of `EffectDescription`
///
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, EnumIter, Serialize, Deserialize)]
pub enum SubEffectType {
    /// Repeats the whole animation periodically
    Repeat,

    /// Changes the rate that the animation plays at
    TimeCurve,

    /// A stop-motion or fitted transformation effect
    TransformPosition
}

///
/// Describes a single sub-effect within a 
///1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubEffectDescription {
    /// The type of this effect
    effect_type: SubEffectType,

    /// How to find this subeffect within the main effect
    address: SmallVec<[usize; 2]>,

    /// The description of this effect
    effect_description: EffectDescription,
}

impl SubEffectType {
    ///
    /// Returns a description of this effect that can be displayed to the user
    ///
    pub fn description(&self) -> &str {
        use self::SubEffectType::*;

        match self {
            Repeat              => "Repeat",
            TimeCurve           => "Time curve",
            TransformPosition   => "Transform position",
        }
    }
}

impl SubEffectDescription {
    ///
    /// Returns the type of this effect
    ///
    pub fn effect_type(&self) -> SubEffectType {
        self.effect_type
    }

    ///
    /// Returns the description for this effect
    ///
    pub fn effect_description(&self) -> &EffectDescription {
        &self.effect_description
    }
}