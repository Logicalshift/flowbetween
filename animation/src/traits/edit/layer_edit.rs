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
    Vector
}

///
/// Represents an edit to a layer
///
#[derive(Clone, PartialEq, Debug)]
pub enum LayerEdit {
    /// Edit to a frame at a specific time
    Paint(Duration, PaintEdit),

    /// Edit to a path at a specific time
    Path(Duration, PathEdit),

    /// Adds a keyframe at a particular point in time
    ///
    /// Edits don't have to correspond to a keyframe - instead, keyframes
    /// indicate where the layer is cleared.
    AddKeyFrame(Duration),

    /// Removes a keyframe previously added at a particular duration
    RemoveKeyFrame(Duration),

    /// Changes the name of this layer
    SetName(String),

    /// Sets this layer so that it is ordered behind the specified layer
    SetOrdering(u64)
}

impl LayerEdit {
    ///
    /// If this edit contains an unassigned element ID, calls the specified function to supply a new
    /// element ID. If the edit already has an ID, leaves it unchanged.
    ///
    pub fn assign_element_id<AssignFn: FnOnce() -> i64>(self, assign_element_id: AssignFn) -> LayerEdit {
        use self::LayerEdit::*;

        match self {
            Paint(when, paint_edit) => Paint(when, paint_edit.assign_element_id(assign_element_id)),
            other                   => other
        }
    }
}
