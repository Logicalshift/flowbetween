use crate::path::*;

// TODO: might be good to have a way to have optional pointers to extra region content that's intended to be added before/after the
// list of paths in this content so we don't always have to copy all of the paths when doing simple things like motions

///
/// Describes what's in a particular animation region
///
pub struct AnimationRegionContent {
    /// The paths tht appear in this region
    pub paths: Vec<AnimationPath>
}
