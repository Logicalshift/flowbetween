use super::super::vector_editor::*;
use super::super::frame_time::*;

use flo_draw::canvas::*;
use flo_scene::*;

use serde::*;

///
/// Messages for the canvas rendering program
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CanvasRender {
    /// The scene is idle (we perform the actual rendering instructions after the scene goes idle)
    Idle,

    /// An update for the canvas has been received
    Update(VectorCanvasUpdate),

    /// Redraw the whole canvas
    Refresh,

    /// Sets the transform to apply to the canvas
    SetTransform(Transform2D),

    /// Sets the time for the canvas
    SetFrame(FrameTime),
}

impl SceneMessage for CanvasRender {
}

///
/// Renders the canvas
///
pub async fn canvas_render_program(input: InputStream<CanvasRender>, context: SceneContext) {

}