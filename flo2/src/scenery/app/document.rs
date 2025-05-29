use flo_scene::*;

use serde::*;
use futures::prelude::*;

use std::sync::*;

///
/// The actions that can be performed on a document in the main app scene
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AppDocumentRequest {

}

impl SceneMessage for AppDocumentRequest {

}

///
/// The 'document' subprogram, one of which runs for each document in the application
///
pub async fn document(document_scene: Arc<Scene>, input: InputStream<AppDocumentRequest>, context: SceneContext) {
    // Run the scene
    let run_scene = document_scene.run_scene_with_threads(4);

    // ... and also run a future listening for document requests from the main app
    future::select(
        run_scene.boxed(),
        async move {
            let mut input = input;
            while let Some(request) = input.next().await {
                let _ = request;
            }
        }.boxed()).await;
}
