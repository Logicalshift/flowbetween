use super::frame_edit::*;

use std::time::Duration;

///
/// Represents an edit to a layer
///
pub enum LayerEdit {
    /// Edit to a frame at a specific time
    Paint(Duration, PaintEdit)
}
