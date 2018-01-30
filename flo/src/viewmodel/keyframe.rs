use std::time::Duration;

///
/// Viewmodel for a keyframe
/// 
#[derive(Clone, PartialEq)]
pub struct KeyFrameViewModel {
    /// When this keyframe occurs relative to the start of the animation
    pub when: Duration,

    /// The layer ID this keyframe is for
    pub layer_id: u64
}
