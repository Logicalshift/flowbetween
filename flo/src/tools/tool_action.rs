use super::brush_preview_action::*;
use super::overlay_action::*;

use flo_animation::*;

use std::sync::*;

///
/// Represents an editing action for a tool
///
#[derive(Debug, PartialEq)]
pub enum ToolAction<ToolData> {
    /// Changes the data that will be specified at the start of the next tool input stream
    Data(ToolData),

    /// Sends some edits to the animation
    EditAnimation(Arc<Vec<AnimationEdit>>),

    /// Invalidates the current frame (forcing it to be redrawn from scratch)
    InvalidateFrame,

    /// Specifies an edit to perform
    Edit(AnimationEdit),

    /// Creates a new key frame if a keyframe is not selected and 'create keyframe on draw' is turned on, creates a new keyframe
    CreateKeyFrameForDrawing,

    /// Specifies a brush preview action to perform
    BrushPreview(BrushPreviewAction),

    /// Sets the tool overlay drawing
    Overlay(OverlayAction),

    /// Clears the current selection
    ClearSelection,

    /// Adds a particular element to the selection
    Select(ElementId)
}
