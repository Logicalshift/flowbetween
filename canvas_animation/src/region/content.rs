use crate::path::*;

///
/// Describes what's in a particular animation region
///
pub struct AnimationRegionContent {
    /// The paths tht appear in this region
    pub paths: Vec<AnimationPath>
}
