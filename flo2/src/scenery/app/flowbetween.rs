use crate::scenery::document::*;
use super::document::*;

use flo_draw::*;
use flo_draw::draw_scene::*;
use flo_draw::canvas::scenery::*;
use flo_scene::*;
use flo_scene::programs::*;
use flo_binding::*;
use futures::prelude::*;
use serde::*;

use std::sync::*;
use std::collections::*;

///
/// Commands for controlling the main flowbetween program
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FlowBetween {
    CreateEmptyDocument(DocumentId),
}

impl SceneMessage for FlowBetween { 
    fn default_target() -> StreamTarget {
        SubProgramId::called("flowbetween::flowbetween").into()
    }
}

///
/// Program that relays events originating from the window in the app scene to the main document in the document scene
///
async fn event_relay_program(drawing_events: impl Unpin + Send + Sink<DocumentRequest>, document_program_id: SubProgramId, input: InputStream<DrawEvent>, context: SceneContext) {
    let mut input           = input;
    let mut drawing_events  = drawing_events;

    while let Some(event) = input.next().await {
        // Interpret some special events
        match &event {
            DrawEvent::Resize(w, h) => {
                // Send a resize request for the resize event
                drawing_events.send(DocumentRequest::Resize(*w as _, *h as _)).await.ok();
            }

            DrawEvent::Closed => {
                // Tell the document to close down when the close request arrives
                drawing_events.send(DocumentRequest::Close).await.ok();

                // Also close down the main document scene when the window is closed
                context.send_message(SceneControl::Close(document_program_id)).await.ok();
            }

            _ => { }
        }

        // Send the event as a normal event
        if drawing_events.send(DocumentRequest::Event(event)).await.is_err() {
            break;
        }
    }
}

///
/// Program that runs in the document scene and sends DrawingWindowRequests out to the application scene (where the actual window lives)
///
async fn drawing_relay_program(drawing_requests: OutputSink<DrawingWindowRequest>, input: InputStream<DrawingWindowRequest>, _context: SceneContext) {
    let mut drawing_requests = drawing_requests;
    let mut input            = input;

    while let Some(request) = input.next().await {
        // We can't wire up 'SendEvents' or similar messages here as they'll send their responses to the app scene, so we ignore them for now
        if let DrawingWindowRequest::SendEvents(_) = &request {
            // TODO: could start an event relay program here instead (we'd have to manage it and stop it when we're done though)
            continue;
        }

        // Pass the event on to the application scene
        let maybe_err = drawing_requests.send(request).await;

        if !maybe_err.is_ok() {
            break;
        }
    }
}

///
/// Creates an empty document in the context
///
async fn create_empty_document(scene: Arc<Scene>, document_program_id: SubProgramId, context: &SceneContext) {
    let properties = WindowProperties::from(&());

    // Create a window for this document
    let render_window_program_id   = SubProgramId::new();
    let drawing_window_program_id  = SubProgramId::new();
    let event_relay_program_id     = SubProgramId::new();

    create_render_window_sub_program(&scene, render_window_program_id, properties.size().get()).unwrap();
    create_drawing_window_program(&scene, drawing_window_program_id, render_window_program_id).unwrap();

    // Each document runs in its own isolated scene (which lets us run subprograms in the scene with their own IDs)
    let document_scene = Arc::new(Scene::default());

    // Add a subprogram in the document scene that relays drawing instructions to the drawing window
    let mut drawing_requests = context.send::<DrawingWindowRequest>(drawing_window_program_id).unwrap();
    document_scene.add_subprogram(subprogram_window(), move |input, context| drawing_relay_program(drawing_requests, input, context), 100);

    // Allow drawing requests to be sent directly to the window
    let drawing_request_filter = FilterHandle::for_filter(|drawing_requests| drawing_requests.map(|req| DrawingWindowRequest::Draw(req)));
    document_scene.connect_programs((), StreamTarget::Filtered(drawing_request_filter, subprogram_window()), StreamId::with_message_type::<DrawingRequest>()).unwrap();

    // Start the main document program within the document scene
    let document_scene_clone = Arc::clone(&document_scene);
    document_scene.add_subprogram(subprogram_flowbetween_document(), move |input, context| flowbetween_document(document_scene_clone, input, context), 20);

    // Add a subprogram to the app scene that relays events from the window to the document scene
    let drawing_events = document_scene.send_to_scene(()).unwrap();
    scene.add_subprogram(event_relay_program_id, move |input, context| event_relay_program(drawing_events, document_program_id, input, context), 20);

    context.send(drawing_window_program_id).unwrap()
        .send(DrawingWindowRequest::SendEvents(event_relay_program_id)).await.ok();

    // Run the document scene in its own subprogram (within the app)
    scene.add_subprogram(document_program_id, move |input, context| document(document_scene, input, context), 1);
}

///
/// Runs the main flowbetween program
///
pub async fn flowbetween(scene: Arc<Scene>, events: InputStream<FlowBetween>, context: SceneContext) {
    let mut events      = events;

    let mut documents = HashMap::new();

    while let Some(evt) = events.next().await {
        use FlowBetween::*;

        match evt {
            CreateEmptyDocument(document_id) => {
                // Create a program ID for the document program
                let document_program_id = SubProgramId::new();

                // Create the document program
                create_empty_document(Arc::clone(&scene), document_program_id, &context).await;

                // Store as in the list of known document programs
                documents.insert(document_id, document_program_id);
            }
        }
    }
}
