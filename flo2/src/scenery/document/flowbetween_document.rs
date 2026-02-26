use super::subprograms::*;
use crate::scenery::document::canvas::*;
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

    /// Request to subscribe to draw events for this document
    SubscribeDrawEvents(StreamTarget),
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

    let mut size            = (1000, 1000);
    let mut draw_size       = (1000.0, 1000.0);
    let mut draw_scale      = 1.0;
    let mut draw_transform  = Transform2D::identity();

    // The document canvas contains the drawing instructions to regenerate the canvas (except for the 'clear canvas' instruction that begins it)
    // We re-use this whenever the document is resized
    let document_canvas = Canvas::new();
    document_canvas.write([
        Draw::ClearCanvas(Color::Rgba(0.8, 0.8, 0.8, 1.0)),
        Draw::CanvasHeight(1024.0),

        Draw::Namespace(NamespaceId::default()),
        Draw::Layer(LayerId(0)),
        Draw::ClearLayer,

        Draw::Namespace(*CANVAS_NAMESPACE),
        Draw::Layer(LayerId(0)),
        Draw::ClearLayer,

        Draw::Namespace(*PHYSICS_LAYER),
        Draw::Layer(LayerId(0)),
        Draw::ClearLayer,

        Draw::Namespace(*DOCK_LAYER),
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

    // Subscribers to events
    let mut draw_event_subscribers  = EventSubscribers::new();

    // Toolbar programs
    document_scene.add_subprogram(subprogram_floating_tools(),  |input, context| floating_tool_dock_program(input, context, LayerId(0)), 20);
    document_scene.add_subprogram(subprogram_tool_dock_left(),  |input, context| tool_dock_program(input, context, DockPosition::Left, LayerId(1), Some(subprogram_floating_tools())), 20);
    document_scene.add_subprogram(subprogram_tool_dock_right(), |input, context| tool_dock_program(input, context, DockPosition::Right, LayerId(2), Some(subprogram_floating_tools())), 20);

    let test_tool       = ToolId::new();
    let test_tool2      = ToolId::new();
    let test_group      = ToolGroupId::new();
    let test_type       = ToolTypeId::new();
    let tool_icon_1     = svg_with_width(include_bytes!("../../../../flo/svg/tools/pencil.svg"), 32.0);
    let tool_icon_2     = svg_with_width(include_bytes!("../../../../flo/svg/tools/ink.svg"), 32.0);

    context.send_message(Tool::CreateTool(test_group, test_type, test_tool)).await.unwrap();
    context.send_message(Tool::CreateTool(test_group, test_type, test_tool2)).await.unwrap();
    context.send_message(Tool::SetToolIcon(test_tool, Arc::new(tool_icon_1.clone()))).await.unwrap();
    context.send_message(Tool::SetToolIcon(test_tool2, Arc::new(tool_icon_2.clone()))).await.unwrap();
    context.send_message(Tool::SetToolLocation(test_tool, subprogram_tool_dock_left().into(), (0.0, 0.0))).await.unwrap();
    context.send_message(Tool::SetToolLocation(test_tool2, subprogram_tool_dock_left().into(), (0.0, 0.1))).await.unwrap();
    context.send_message(Tool::Select(test_tool)).await.unwrap();

    // Set up an initial canvas: make sure the SqliteCanvas and the rendering program both exist
    context.send::<SqliteCanvasRequest>(()).unwrap();
    context.send::<CanvasRender>(()).unwrap();

    context.send_message(CanvasRender::SetTransform(Transform2D::scale(1.5, 1.5))).await.unwrap();

    document_scene.add_subprogram(ShapeType::default().render_program_id(), standard_shape_type_renderer_program, 10);

    // Add an ellipse to the canvas
    let layer_1  = vector_add_layer(&[&Name::from("Layer 1")]).await;
    let _ellipse = vector_add_shape(
        ShapeType::default(), 
        CanvasShape::Ellipse(CanvasEllipse { min: CanvasPoint { x: 960.0-150.0, y: 540.0-50.0 }, max: CanvasPoint { x: 960.0+150.0, y: 540.0+50.0 }, direction: CanvasPoint { x: 0.0, y: 1.0 } }),
        (layer_1, FrameTime::ZERO),
        &[
            &FlatFill(Color::Rgba(0.0, 0.5, 1.0, 1.0)),
            &Stroke(StrokeWidth(2.0), LineCap::Round, LineJoin::Round, Color::Rgba(0.0, 0.0, 0.0, 1.0))
        ],
        vec![]).await;

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

                        DrawEvent::Scale(new_scale)     => { draw_scale = *new_scale; }
                        DrawEvent::Resize(w, h)         => { draw_size = (*w, *h); }
                        DrawEvent::CanvasTransform(t)   => { draw_transform = *t; }

                        _ => { }
                    }

                    // Send to the focus program
                    focus.send(Focus::Event(event.clone())).await.ok();
                    draw_event_subscribers.send(event).await;
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

                DocumentRequest::SubscribeDrawEvents(target) => {
                    if let Ok(mut draw_events) = context.send(target.clone()) {
                        // Indicate the current size of the drawing region
                        draw_events.send(DrawEvent::Resize(draw_size.0, draw_size.1)).await.ok();
                        draw_events.send(DrawEvent::Scale(draw_scale)).await.ok();
                        draw_events.send(DrawEvent::CanvasTransform(draw_transform)).await.ok();

                        // Subscribe to the draw events
                        draw_event_subscribers.subscribe(&context, target);
                    }
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
