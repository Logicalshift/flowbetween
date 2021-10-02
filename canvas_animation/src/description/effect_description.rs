use super::geometry::*;

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

    /// A simple move through a bezier path at constant speed
    Move(Duration, BezierPath)
}
