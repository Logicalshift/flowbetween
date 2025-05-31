use super::subprograms::*;

use flo_draw::*;
use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;
use flo_scene::*;
use flo_scene::programs::*;

use futures::prelude::*;
use serde::*;

use std::sync::*;

///
/// Request for the main document program
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DocumentRequest {
    /// Indicates that the pending requests have resolved themselves
    Idle,

    /// Resizes the document window
    Resize(usize, usize),

    /// Renders some items to the drawing window
    Draw(DrawingRequest),

    /// Event has occurred on the window
    Event(DrawEvent),

    /// Indicates that this document is being closed
    Close,
}

impl SceneMessage for DocumentRequest {
    fn default_target() -> StreamTarget {
        subprogram_flowbetween_document().into()
    }
}

///
/// The main document subprogram (runs a flowbetween document window)
///
pub async fn flowbetween_document(document_scene: Arc<Scene>, input: InputStream<DocumentRequest>, context: SceneContext) {
    let program_id = context.current_program_id().unwrap();

    // Set up to receive idle events and drawing requests
    document_scene.connect_programs((), program_id, StreamId::with_message_type::<DocumentRequest>()).unwrap();
    document_scene.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|stream| stream.map(|msg: IdleNotification| DocumentRequest::Idle))), (), StreamId::with_message_type::<IdleNotification>()).unwrap();
    document_scene.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|stream| stream.map(|msg| DocumentRequest::Draw(msg)))), (), StreamId::with_message_type::<DrawingRequest>()).unwrap();
    document_scene.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|stream| stream.map(|msg| DocumentRequest::Draw(msg)))), program_id, StreamId::with_message_type::<DrawingRequest>()).unwrap();

    context.send_message(IdleRequest::WhenIdle(program_id)).await.ok();

    // Set up the window to its initial state
    let mut window_drawing  = context.send::<DrawingRequest>(subprogram_window()).unwrap();
    let mut window_setup    = vec![];

    window_setup.clear_canvas(Color::Rgba(0.8, 0.8, 0.8, 1.0));
    window_setup.canvas_height(1000.0);
    window_setup.center_region(0.0, 0.0, 1000.0, 1000.0);

    window_drawing.send(DrawingRequest::Draw(Arc::new(window_setup))).await.ok();

    let mut size = (1000, 1000);

    // The document canvas contains the drawing instructions to regenerate the canvas (except for the 'clear canvas' instruction that begins it)
    // We re-use this whenever the document is resized
    let document_canvas = Canvas::new();
    document_canvas.write([Draw::ClearCanvas(Color::Rgba(0.8, 0.8, 0.8, 1.0))].into_iter().collect());

    // Drawing instructions that are waiting for this document scene to become idle
    let mut pending_drawing = Vec::with_capacity(128);

    // TODO: start the other document subprograms

    // Process the events for this document (in chunks, so we group together multiple drawing or resize requests if they occur)
    let mut input = input.ready_chunks(100);
    while let Some(many_requests) = input.next().await {
        let mut new_size = size;

        // Process the buffered requests
        for request in many_requests {
            match request {
                DocumentRequest::Draw(DrawingRequest::Draw(drawing)) => {
                    // Write to the canvas
                    document_canvas.write(drawing.iter().cloned().collect());

                    // Also write to the pending drawing instructions, ready to pass on to the main window
                    for draw in drawing.iter() {
                        match draw {
                            Draw::ClearCanvas(background) => {
                                pending_drawing = vec![draw.clone()];
                            }

                            _ => {
                                pending_drawing.push(draw.clone());
                            }
                        }
                    }
                }

                DocumentRequest::Resize(width, height) => {
                    if new_size != (width, height) {
                        // Update the size of the canvas
                        new_size = (width, height);

                        // Reset the pending drawing to the canvas contents
                        pending_drawing = document_canvas.get_drawing();

                        // Set up the canvas display region after the point that the canvas is cleared
                        let clear_canvas_idx = pending_drawing.iter().position(|draw| matches!(draw, Draw::ClearCanvas(_)));

                        if let Some(clear_canvas_idx) = clear_canvas_idx {
                            pending_drawing.splice(clear_canvas_idx+1..clear_canvas_idx+1, [
                                Draw::CanvasHeight(height as _),
                                Draw::CenterRegion((0.0, 0.0), (width as _, height as _)),
                            ]);
                        }
                    }
                }

                DocumentRequest::Event(event) => {
                    println!("{:?}", event);
                }

                DocumentRequest::Idle => {
                    // Send any pending drawing instructions whenever the scene processing stops
                    if !pending_drawing.is_empty() {
                        use std::mem;

                        let mut recent_drawing = Vec::with_capacity(128);
                        mem::swap(&mut recent_drawing, &mut pending_drawing);

                        window_drawing.send(DrawingRequest::Draw(Arc::new(recent_drawing))).await.ok();
                    }
                }

                DocumentRequest::Close => {
                    // When the document is closed, we stop the whole scene
                    context.send_message(SceneControl::StopScene).await.ok();
                }
            }
        }

        // If the requests included a window size change, flush the pending drawing instructions early
        if new_size != size {
            size = new_size;

            use std::mem;

            let mut recent_drawing = Vec::with_capacity(128);
            mem::swap(&mut recent_drawing, &mut pending_drawing);

            window_drawing.send(DrawingRequest::Draw(Arc::new(recent_drawing))).await.ok();
        }
    }
}
