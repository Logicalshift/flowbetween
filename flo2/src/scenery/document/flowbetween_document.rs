use super::subprograms::*;

use flo_draw::*;
use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;
use flo_scene::*;

use futures::prelude::*;
use serde::*;

use std::sync::*;

///
/// Request for the main document program
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DocumentRequest {

}

impl SceneMessage for DocumentRequest {
    fn default_target() -> StreamTarget {
        subprogram_flowbetween_document().into()
    }
}

///
/// The main document subprogram
///
pub async fn flowbetween_document(input: InputStream<DocumentRequest>, context: SceneContext) {
    // Set up the window to its initial state
    let mut window_drawing  = context.send::<DrawingRequest>(subprogram_window()).unwrap();
    let mut window_setup    = vec![];

    window_setup.clear_canvas(Color::Rgba(0.8, 0.8, 0.8, 1.0));
    window_setup.canvas_height(1000.0);
    window_setup.center_region(0.0, 0.0, 1000.0, 1000.0);

    window_drawing.send(DrawingRequest::Draw(Arc::new(window_setup))).await.ok();

    // TODO: start the other document subprograms

    // Process the events for this document
    let mut input = input;
    while let Some(request) = input.next().await {

    }
}
