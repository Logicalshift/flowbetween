use crate::scenery::document::*;

use flo_scene::*;

use serde::*;
use futures::prelude::*;

use std::sync::*;

///
/// The actions that can be performed on a document in the main app scene
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AppDocumentRequest {
    ///
    /// Sends a document request to the target scene
    ///
    DocumentRequest(DocumentRequest),
}

impl SceneMessage for AppDocumentRequest {

}

///
/// The 'document' subprogram, one of which runs for each document in the application
///
pub async fn document(document_scene: Arc<Scene>, input: InputStream<AppDocumentRequest>, _context: SceneContext) {
    // Run the scene
    let run_scene = document_scene.run_scene_with_threads(4);

    // Create stream for document requests
    let mut document_requests = document_scene.send_to_scene(()).unwrap();

    use flo_draw::canvas::*;
    use flo_draw::canvas::scenery::*;
    let mut test_drawing = vec![];
    test_drawing.new_path();
    test_drawing.circle(500.0, 500.0, 250.0);
    test_drawing.fill_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
    test_drawing.fill();
    document_requests.send(DocumentRequest::Draw(DrawingRequest::Draw(Arc::new(test_drawing)))).await.unwrap();

    // ... and also run a future listening for document requests from the main app
    future::select(
        run_scene.boxed(),
        async move {
            let mut input = input;
            while let Some(request) = input.next().await {
                match request {
                    AppDocumentRequest::DocumentRequest(req) => {
                        document_requests.send(req).await.ok();
                    }
                }
            }
        }.boxed()).await;

    // When shutting down the program, also shut down the document if it's still there
    // TODO: ... and the window, if it's still there
    if let Ok(mut document_requests) = document_scene.send_to_scene(()) {
        document_requests.send(DocumentRequest::Close).await.ok();
    }
}
