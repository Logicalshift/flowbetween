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
    /// Custom JSON effect
    Other,

    /// Repeats the whole animation periodically
    Repeat,

    /// Changes the rate that the animation plays at
    TimeCurve,

    /// Follows a curve at a constant speed
    LinearPosition,

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
            Other               => "Custom effect",
            Repeat              => "Repeat",
            TimeCurve           => "Time curve",
            LinearPosition      => "Move at constant speed",
            TransformPosition   => "Transform position",
        }
    }
}

impl SubEffectDescription {
    ///
    /// Creates a new effect description
    ///
    fn new(effect_type: SubEffectType, address: SmallVec<[usize; 2]>, effect_description: &EffectDescription) -> SubEffectDescription {
        SubEffectDescription {
            effect_type:            effect_type,
            address:                address,
            effect_description:     effect_description.clone()
        }
    }

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

impl EffectDescription {
    ///
    /// Returns the sub-effects that make up this animation effect
    ///
    pub fn sub_effects(&self) -> Vec<SubEffectDescription> {
        // Build the sub-effects for this effect description
        let mut result = vec![];
        self.build_sub_effects(smallvec![], &mut result);

        result
    }

    ///
    /// Adds the sub-effects for this effect description to a list
    ///
    fn build_sub_effects(&self, address: SmallVec<[usize; 2]>, sub_effects: &mut Vec<SubEffectDescription>) {
        use self::EffectDescription::*;

        match self {
            FrameByFrameReplaceWhole                => { /* Base animation type */ }
            FrameByFrameAddToInitial                => { /* Base animation type */ }

            Other(_name, _json)                     => { sub_effects.push(SubEffectDescription::new(SubEffectType::Other, address, self)); }
            Move(_length, _path)                    => { sub_effects.push(SubEffectDescription::new(SubEffectType::LinearPosition, address, self)); }
            FittedTransform(_origin, _points)       => { sub_effects.push(SubEffectDescription::new(SubEffectType::TransformPosition, address, self)); }
            StopMotionTransform(_origin, _points)   => { sub_effects.push(SubEffectDescription::new(SubEffectType::TransformPosition, address, self)); }

            Repeat(_length, effect)                 => {
                // We assume 'repeat' and 'time curve' apply to the effect as a whole and not a partial sub-effect at the moment: an improvement might be to support representing a tree of effects here
                let mut new_address = address.clone();
                new_address.push(0);

                sub_effects.push(SubEffectDescription::new(SubEffectType::Repeat, address, self));
                effect.build_sub_effects(new_address, sub_effects);
            }
            TimeCurve(_curve, effect)               => {
                // We assume 'repeat' and 'time curve' apply to the effect as a whole and not a partial sub-effect at the moment: an improvement might be to support representing a tree of effects here
                let mut new_address = address.clone();
                new_address.push(0);

                sub_effects.push(SubEffectDescription::new(SubEffectType::TimeCurve, address, self));
                effect.build_sub_effects(new_address, sub_effects);
            }

            Sequence(effects)                       => {
                // Sequence just adds all of the effects one after the other
                for (idx, effect) in effects.iter().enumerate() {
                    let mut new_address = address.clone();
                    new_address.push(idx);

                    effect.build_sub_effects(new_address, sub_effects);
                }
            }
        }
    }
}
