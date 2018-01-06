use super::frame_edit::*;

use std::time::Duration;

///
/// Represents a type of layer edit
/// 
/// Layers may have different types, so this can be used to check what
/// types of action a particular layer might support.
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum LayerEditType {
    Paint
}

///
/// Represents an edit to a layer
///
#[derive(Clone, PartialEq, Debug)]
pub enum LayerEdit {
    /// Edit to a frame at a specific time
    Paint(Duration, PaintEdit),

    /// Adds a keyframe at a particular point in time
    /// 
    /// Edits don't have to correspond to a keyframe - instead, keyframes
    /// indicate where the layer is cleared.
    AddKeyFrame(Duration),

    /// Removes a keyframe previously added at a particular duration
    RemoveKeyFrame(Duration)
}
