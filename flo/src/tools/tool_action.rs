use super::brush_preview_action::*;
use super::overlay_action::*;

use flo_animation::*;

///
/// Represents an editing action for a tool
///
#[derive(Debug)]
pub enum ToolAction<ToolData> {
    /// Changes the data that will be specified at the start of the next tool input stream
    Data(ToolData),

    /// Invalidates the current frame (forcing it to be redrawn from scratch)
    InvalidateFrame,

    /// Specifies an edit to perform
    Edit(AnimationEdit),

    /// Specifies a brush preview action to perform
    BrushPreview(BrushPreviewAction),

    /// Sets the tool overlay drawing
    Overlay(OverlayAction),

    /// Clears the current selection
    ClearSelection,

    /// Adds a particular element to the selection
    Select(ElementId)
}
