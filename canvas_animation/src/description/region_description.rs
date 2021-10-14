use super::space::*;
use super::effect_description::*;

use serde::{Serialize, Deserialize};

///
/// Describes a region and the effect that applies to it
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegionDescription(pub Vec<BezierPath>, pub EffectDescription);
