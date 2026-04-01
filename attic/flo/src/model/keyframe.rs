use std::time::Duration;

///
/// Viewmodel for a keyframe
///
#[derive(Clone, PartialEq)]
pub struct KeyFrameModel {
    /// When this keyframe occurs relative to the start of the animation
    pub when: Duration,

    /// The frame number this keyframe occurs on
    pub frame: u32,

    /// The layer ID this keyframe is for
    pub layer_id: u64
}
