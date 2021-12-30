use super::effect_description::*;

///
/// The 'base' animation for an animation region
///
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, EnumIter)]
pub enum BaseAnimationType {
    /// Build over time animations (later drawings are added to the existing ones, the default behaviour if no animation is defined)
    BuildOverTime,

    /// Frame-by-frame animations (later drawings replace the existing ones)
    FrameByFrame,

    /// The first frame is always redrawn and future frames are added to it
    BuildOnFirstFrame,
}

impl BaseAnimationType {
    ///
    /// Generates a description of this animation type
    ///
    pub fn description(&self) -> &str {
        use self::BaseAnimationType::*;

        match self {
            FrameByFrame        => { "Frame-by-frame" }
            BuildOverTime       => { "Build over time" }
            BuildOnFirstFrame   => { "Draw over initial frame" }
        }
    }
}

impl EffectDescription {
    ///
    /// Returns the base animation type for an effect description
    ///
    pub fn base_animation_type(&self) -> BaseAnimationType {
        use self::EffectDescription::*;

        match self {
            // The 'frame by frame' animation types override the usual 'build over time' effect
            FrameByFrameReplaceWhole    => BaseAnimationType::FrameByFrame,
            FrameByFrameAddToInitial    => BaseAnimationType::BuildOnFirstFrame,

            // In sequences, the first element determines the base animation type
            Sequence(sequence)          => { if sequence.len() > 0 { sequence[0].base_animation_type() } else { BaseAnimationType::BuildOverTime } }
            
            // Embedded effects like repeats or time curves preserve the base animation type of their underlying animation
            Repeat(_length, effect)     => { effect.base_animation_type() },
            TimeCurve(_curve, effect)   => { effect.base_animation_type() },

            // Other built-in effects mean there's no 'base' type, ie we're using the build over time effect
            Other(_, _)                 |
            Move(_, _)                  |
            FittedTransform(_, _)       |
            StopMotionTransform(_, _)   => BaseAnimationType::BuildOverTime
        }
    }

    ///
    /// Creates a new effect description for a new base animation type
    ///
    pub fn update_effect_animation_type(&self, new_base_type: BaseAnimationType) -> EffectDescription {
        use self::EffectDescription::*;

        // Work out the new base description element
        let new_description = match new_base_type {
            BaseAnimationType::FrameByFrame         => Some(FrameByFrameReplaceWhole),
            BaseAnimationType::BuildOnFirstFrame    => Some(FrameByFrameAddToInitial),
            BaseAnimationType::BuildOverTime        => None
        };

        match self {
            // Basic frame-by-frame items are replaced with sequences
            FrameByFrameReplaceWhole    |
            FrameByFrameAddToInitial    => Sequence(new_description.into_iter().collect()),

            // Sequences update the first element (or insert a new first element if there's no base type there)
            Sequence(sequence)          => {
                if sequence.len() == 0 {
                    // Empty sequence is just replaced with the new base description
                    Sequence(new_description.into_iter().collect())
                } else {
                    // First sequence element is replaced if it's already a base animation type, otherwise the base type is added to the start of the sequence
                    match sequence[0] {
                        FrameByFrameReplaceWhole | FrameByFrameAddToInitial => Sequence(new_description.into_iter().chain(sequence.iter().skip(1).cloned()).collect()),
                        _                                                   => Sequence(new_description.into_iter().chain(sequence.iter().cloned()).collect())
                    }
                }
            }

            // Embedded effects recurse
            Repeat(length, effect)      => { Repeat(*length, Box::new(effect.update_effect_animation_type(new_base_type))) },
            TimeCurve(curve, effect)    => { TimeCurve(curve.clone(), Box::new(effect.update_effect_animation_type(new_base_type))) },

            // Other effects are unaffected
            Other(_, _)                 |
            Move(_, _)                  |
            FittedTransform(_, _)       |
            StopMotionTransform(_, _)   => self.clone()
        }
    }
}
