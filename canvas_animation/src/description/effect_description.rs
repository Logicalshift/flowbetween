use super::time::*;
use super::space::*;

use std::time::*;

use serde::{Serialize, Deserialize};
use serde_json as json;

///
/// Describes an animation effect that can be constructed later on
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EffectDescription {
    /// Description of an effect with custom deserialization
    Other(String, json::Value),

    /// Applies a series of effects in sequence
    Sequence(Vec<EffectDescription>),

    /// Repeats another animation effect from the start
    Repeat(Duration, Box<EffectDescription>),

    /// Applies a time curve to another animation effect
    TimeCurve(Vec<(f64, f64, f64)>, Box<EffectDescription>),

    /// Animate frame-by-frame by replacing the entire region
    FrameByFrameReplaceWhole,

    /// Animate frame-by-frame by adding new frames to the time=0 frame
    FrameByFrameAddToInitial,

    /// A simple move through a bezier path at constant speed
    Move(Duration, BezierPath),

    /// Transform through a set of points, interpolating at other times using a fitted curve
    ///
    /// Contents are an anchor point and positions for the animation
    FittedTransform(Point2D, Vec<TimeTransformPoint>),

    /// Transform through a set of points with no interpolation
    ///
    /// Contents are an anchor point and positions for the animation
    StopMotionTransform(Point2D, Vec<TimeTransformPoint>)
}

impl EffectDescription {
    ///
    /// Puts the effect description in a box
    ///
    pub fn boxed(self) -> Box<EffectDescription> {
        Box::new(self)
    }

    ///
    /// Changes this effect description to a sequence of 1
    ///
    pub fn sequence(self) -> EffectDescription {
        match self {
            EffectDescription::Sequence(_)  => self,
            _                               => EffectDescription::Sequence(vec![self])
        }
    }
}
