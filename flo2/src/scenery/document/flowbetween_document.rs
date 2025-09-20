use super::subprograms::*;
use crate::scenery::ui::*;

use flo_draw::*;
use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;
use flo_scene::*;
use flo_scene::programs::*;
use flo_binding::*;

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

    fn initialise(init_context: &impl SceneInitialisationContext) {
        // Set up to receive idle events and drawing requests
        init_context.connect_programs((), subprogram_flowbetween_document(), StreamId::with_message_type::<DocumentRequest>()).unwrap();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|stream| stream.map(|_msg: IdleNotification| DocumentRequest::Idle))), (), StreamId::with_message_type::<IdleNotification>()).unwrap();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|stream| stream.map(|msg| DocumentRequest::Draw(msg)))), (), StreamId::with_message_type::<DrawingRequest>()).unwrap();
    }
}

///
/// The main document subprogram (runs a flowbetween document window)
///
pub async fn flowbetween_document(document_scene: Arc<Scene>, input: InputStream<DocumentRequest>, context: SceneContext) {
    let program_id = context.current_program_id().unwrap();

    document_scene.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|stream| stream.map(|msg| DocumentRequest::Draw(msg)))), program_id, StreamId::with_message_type::<DrawingRequest>()).unwrap();

    // (TEST: start the physics layer)
    context.send_message(PhysicsLayer::UpdatePosition(PhysicsToolId::new())).await.ok();

    // Set up the window to its initial state
    let mut idle_requests   = context.send::<IdleRequest>(()).unwrap();
    let mut window_drawing  = context.send::<DrawingRequest>(()).unwrap();
    let mut dialog          = context.send::<Dialog>(()).unwrap();
    let mut focus           = context.send::<Focus>(()).unwrap();
    let mut window_setup    = vec![];

    window_setup.clear_canvas(Color::Rgba(0.8, 0.8, 0.8, 1.0));
    window_setup.canvas_height(1000.0);
    window_setup.transform(Transform2D::scale(1.0, -1.0));
    window_setup.center_region(0.0, 0.0, 1000.0, 1000.0);

    window_drawing.send(DrawingRequest::Draw(Arc::new(window_setup))).await.ok();

    let mut size = (1000, 1000);

    // Vague testing of the dialog system
    let dialog1 = DialogId::new();
    let dialog2 = DialogId::new();
    dialog.send(Dialog::CreateDialog(dialog1, program_id, (UiPoint(200.0, 200.0), UiPoint(400.0, 400.0)))).await.ok();
    dialog.send(Dialog::CreateDialog(dialog2, program_id, (UiPoint(600.0, 600.0), UiPoint(800.0, 800.0)))).await.ok();
    dialog.send(Dialog::AddControl(dialog1, ControlId::new(), (UiPoint(10.0, 10.0), UiPoint(180.0, 50.0)), ControlType::Label(bind("Label 1".to_string()).into()), ControlValue::None)).await.ok();
    dialog.send(Dialog::AddControl(dialog2, ControlId::new(), (UiPoint(10.0, 10.0), UiPoint(180.0, 50.0)), ControlType::Label(bind("Label 2".to_string()).into()), ControlValue::None)).await.ok();

    // The document canvas contains the drawing instructions to regenerate the canvas (except for the 'clear canvas' instruction that begins it)
    // We re-use this whenever the document is resized
    let document_canvas = Canvas::new();
    document_canvas.write([
        Draw::ClearCanvas(Color::Rgba(0.8, 0.8, 0.8, 1.0)),
        Draw::CanvasHeight(1024.0),

        Draw::Namespace(NamespaceId::default()),
        Draw::Layer(LayerId(0)),
        Draw::ClearLayer,

        Draw::Namespace(*PHYSICS_LAYER),
        Draw::Layer(LayerId(0)),
        Draw::ClearLayer,

        Draw::Namespace(*DIALOG_LAYER),
        Draw::Layer(LayerId(0)),
        Draw::ClearLayer,

        Draw::Namespace(NamespaceId::default()),
        Draw::Layer(LayerId(0)),
    ].into_iter().collect());

    // Drawing instructions that are waiting for this document scene to become idle
    let mut pending_drawing         = Vec::with_capacity(128);
    let mut waiting_for_new_frame   = false;
    let mut waiting_for_idle        = false;

    // TODO: start the other document subprograms

    document_scene.add_subprogram(SubProgramId::new(), |input: InputStream<TimeOut>, context| async move {
        use std::time::{Duration, Instant};

        context.send_message(TimerRequest::CallEvery(context.current_program_id().unwrap(), 1, Duration::from_millis(16))).await.ok();

        let mut input = input;
        let mut drawing = context.send::<DrawingRequest>(()).unwrap();

        let start = Instant::now();

        while let Some(TimeOut(_, _)) = input.next().await {
            let when = Instant::now().duration_since(start);

            let t = when.as_millis() as f64;
            let t = t / 2000.0;

            let mut circle = vec![];
            circle.layer(LayerId(1));
            circle.clear_layer();
            circle.new_path();
            circle.circle((t.sin()*400.0 + 500.0) as f32, ((t*0.32).cos()*400.0 + 500.0) as f32, 100.0);
            circle.fill_color(Color::Rgba(0.4, 0.4, 0.8, 1.0));
            circle.fill();

            drawing.send(DrawingRequest::Draw(Arc::new(circle))).await.ok();
        }
    }, 1);

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
                                pending_drawing = vec![Draw::ClearCanvas(*background)];
                            }

                            _ => {
                                pending_drawing.push(draw.clone());
                            }
                        }
                    }

                    // Request an idle message (we'll draw once everything is idle)
                    if !waiting_for_idle {
                        idle_requests.send(IdleRequest::WhenIdle(program_id)).await.ok();
                        waiting_for_idle = true;
                    }
                }

                DocumentRequest::Resize(width, height) => {
                    if new_size != (width, height) {
                        // Update the size of the canvas
                        new_size = (width, height);

                        /*
                        // Reset the pending drawing to the canvas contents
                        pending_drawing = document_canvas.get_drawing();

                        // Set up the canvas display region after the point that the canvas is cleared
                        let clear_canvas_idx = pending_drawing.iter().position(|draw| matches!(draw, Draw::ClearCanvas(_)));

                        if let Some(clear_canvas_idx) = clear_canvas_idx {
                            pending_drawing.splice(clear_canvas_idx+1..clear_canvas_idx+1, [
                                Draw::CanvasHeight(height as _),
                                Draw::MultiplyTransform(Transform2D::scale(1.0, -1.0)),
                                Draw::CenterRegion((0.0, 0.0), (width as _, height as _)),
                            ]);
                        }
                        */
                    }
                }

                DocumentRequest::Event(event) => {
                    // Process the event
                    match &event {
                        DrawEvent::NewFrame => {
                            waiting_for_new_frame = false;

                            if !pending_drawing.is_empty() && !waiting_for_idle {
                                // Request an idle event if there's more to draw waiting
                                idle_requests.send(IdleRequest::WhenIdle(program_id)).await.ok();
                                waiting_for_idle = true;
                            }
                        }

                        _ => { }
                    }

                    // Send to the focus program
                    focus.send(Focus::Event(event)).await.ok();
                }

                DocumentRequest::Idle => {
                    // No longer expecting an idle request
                    waiting_for_idle = false;

                    // Send any pending drawing instructions whenever the scene processing stops
                    if !pending_drawing.is_empty() && !waiting_for_new_frame {
                        use std::mem;

                        let mut recent_drawing = Vec::with_capacity(128);
                        mem::swap(&mut recent_drawing, &mut pending_drawing);

                        window_drawing.send(DrawingRequest::Draw(Arc::new(recent_drawing))).await.ok();

                        waiting_for_new_frame = true;
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

            if !waiting_for_new_frame {
                use std::mem;

                let mut recent_drawing = Vec::with_capacity(128);
                mem::swap(&mut recent_drawing, &mut pending_drawing);

                window_drawing.send(DrawingRequest::Draw(Arc::new(recent_drawing))).await.ok();

                waiting_for_new_frame = true;
            }
        }
    }
}
