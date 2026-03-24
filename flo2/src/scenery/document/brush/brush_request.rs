use super::brush_response::*;

use crate::scenery::document::canvas::*;

use flo_scene::*;
use flo_scene::programs::*;

use serde::*;

///
/// A request to run a brush command
///
/// Brushes work as a pipeline of commands, converting from raw brush inputs to canvas shapes
///
#[derive(Serialize, Deserialize)]
pub enum BrushRequest {
    /// Requests to start a new brush stroke. The supplied stream target recieves the brush responses.
    ///
    /// Brush responses don't directly provide the shapes for the brush but rather a pipeline that can be used to process
    /// the raw brush points into a brush stroke.
    RunBrush(CanvasBrushId, StreamTarget),
}

impl SceneMessage for BrushRequest {
}

impl QueryRequest for BrushRequest {
    type ResponseData = BrushResponse;

    fn with_new_target(self, new_target: StreamTarget) -> Self {
        match self {
            Self::RunBrush(brush_id, _) => Self::RunBrush(brush_id, new_target)
        }
    }
}
