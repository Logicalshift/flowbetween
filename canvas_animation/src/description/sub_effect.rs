use super::space::*;
use super::effect_description::*;

use smallvec::*;
use serde::{Serialize, Deserialize};
use serde_json as json;

use std::iter;
use std::time::{Duration};

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

    ///
    /// Creates a default effect description for this effect
    ///
    pub fn default_effect_description(&self) -> EffectDescription {
        use self::SubEffectType::*;

        match self {
            Other               => EffectDescription::Other("".to_string(), json::Value::Null),
            Repeat              => EffectDescription::Repeat(Duration::from_millis(1000), EffectDescription::Sequence(vec![]).boxed()),
            TimeCurve           => EffectDescription::TimeCurve(vec![], EffectDescription::Sequence(vec![]).boxed()),
            LinearPosition      => EffectDescription::Move(Duration::from_millis(1000), BezierPath(Point2D(0.0, 0.0), vec![])),
            TransformPosition   => EffectDescription::FittedTransform(Point2D(0.0, 0.0), vec![])
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
    /// Returns the address of this effect
    ///
    /// This uniquely identifies the effect relative to the parent animation effect description it comes from
    ///
    pub fn address(&self) -> Vec<usize> {
        self.address.iter().cloned().collect()
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

    ///
    /// Finds a sub-effect within this effect
    ///
    fn find_sub_effect(&mut self, address: &SmallVec<[usize; 2]>) -> Option<&mut EffectDescription> {
        if address.len() == 0 {
            // No address = we've found the effect
            Some(self)
        } else {
            use self::EffectDescription::*;

            // Effect should have a recursive portion
            match self {
                Other(_, _)                 |
                FrameByFrameReplaceWhole    |
                FrameByFrameAddToInitial    |
                Move(_, _)                  |
                FittedTransform(_, _)       |
                StopMotionTransform(_, _)   => None,

                Sequence(seq)               => {
                    let new_address = address.iter().skip(1).cloned().collect();
                    seq[address[0]].find_sub_effect(&new_address)
                }
                Repeat(_, subeffect)        |
                TimeCurve(_, subeffect)     => {
                    let new_address = address.iter().skip(1).cloned().collect();
                    subeffect.find_sub_effect(&new_address)
                }
            }
        }
    }

    ///
    /// If this effect is applied to another animation effect, this returns the effect that this applies to
    ///
    /// Returns the empty sequence for non-recursive effects
    ///
    pub fn recursive_effect(&self) -> EffectDescription {
        use self::EffectDescription::*;

        match self {
            Other(_, _)                 |
            FrameByFrameReplaceWhole    |
            FrameByFrameAddToInitial    |
            Move(_, _)                  |
            FittedTransform(_, _)       |
            StopMotionTransform(_, _)   |
            Sequence(_)                 => EffectDescription::Sequence(vec![]),

            Repeat(_, subeffect)        |
            TimeCurve(_, subeffect)     => (**subeffect).clone()
        }
    }

    ///
    /// Replaces a sub-effect with a new effect description
    ///
    /// The sub-effect description must have been generated from this effect description by calling sub_effects (the way
    /// it describes its location is only valid for this effect)
    ///
    /// For recursive effects, this will ignore the effect defined in the new effect and keep the original effect.
    /// For example, if the new effect is a 'repeat' and the original effect is also a 'repeat', the orignal repeated
    /// effect will be kept (and the new one ignored). This makes it possible to store edits to these recursive effects
    /// without having to replace the whole effects tree.
    ///
    /// In the event the original effect is not recursive, this will use the empty sequence as the content of the new
    /// recursive portion of the effect.
    ///
    pub fn replace_sub_effect(&self, old_effect: &SubEffectDescription, new_effect: EffectDescription) -> EffectDescription {
        // Find the effect that's being replaced
        let mut new_description = self.clone();
        let replaced_effect     = new_description.find_sub_effect(&old_effect.address);
        let replaced_effect     = if let Some(replaced_effect) = replaced_effect { replaced_effect } else { return new_description; };

        // Replace any recursive portion of the new effect
        use self::EffectDescription::*;

        let new_effect = match new_effect {
            Other(_, _)                 |
            FrameByFrameReplaceWhole    |
            FrameByFrameAddToInitial    |
            Move(_, _)                  |
            FittedTransform(_, _)       |
            StopMotionTransform(_, _)   |
            Sequence(_)                 => new_effect,

            Repeat(len, _)              => Repeat(len, replaced_effect.recursive_effect().boxed()),
            TimeCurve(curve, _)         => TimeCurve(curve, replaced_effect.recursive_effect().boxed()),
        };

        // Replace the effect
        *replaced_effect        = new_effect;

        // The new description is the result
        new_description
    }

    ///
    /// Adds a new effect to this effect description
    ///
    pub fn add_new_effect(&self, new_effect_type: SubEffectType) -> EffectDescription {
        use self::EffectDescription::*;

        // Create the new effect with default parameters
        let new_effect = new_effect_type.default_effect_description();

        match (new_effect, self) {
            // Nested effects have special behaviour
            (Repeat(_, _), Repeat(_, _))            => self.clone(),
            (TimeCurve(_, _), TimeCurve(_, _))      => self.clone(),
            (_, Repeat(duration, nested_effect))    => Repeat(*duration, nested_effect.add_new_effect(new_effect_type).boxed()),
            (Repeat(duration, _), _)                => Repeat(duration, self.clone().boxed()),      // Note ordering: Repeat will nest over timecurve so it has priority
            (_, TimeCurve(curve, nested_effect))    => TimeCurve(curve.clone(), nested_effect.add_new_effect(new_effect_type).boxed()),
            (TimeCurve(curve, _), _)                => TimeCurve(curve, self.clone().boxed()),

            // Sequences are extended
            (new_effect, Sequence(items))           => Sequence(items.iter().cloned().chain(iter::once(new_effect)).collect()),

            // Standard effects get turned into a sequence
            (new_effect, Other(_, _))               |
            (new_effect, FrameByFrameReplaceWhole)  |
            (new_effect, FrameByFrameAddToInitial)  |
            (new_effect, Move(_, _))                |
            (new_effect, FittedTransform(_, _))     |
            (new_effect, StopMotionTransform(_, _)) => Sequence(vec![self.clone(), new_effect])
        }
    }
}
