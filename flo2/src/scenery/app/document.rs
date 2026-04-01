// FlowBetween, a tool for creating vector animations
// Copyright (C) 2026 Andrew Hunter
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
