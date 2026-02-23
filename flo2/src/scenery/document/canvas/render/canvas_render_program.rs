use super::super::frame_time::*;
use super::super::layer::*;
use super::super::vector_editor::*;

use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;
use flo_scene::*;
use flo_scene::programs::*;

use futures::prelude::*;
use serde::*;
use once_cell::sync::{Lazy};

use std::collections::*;

pub static CANVAS_NAMESPACE: Lazy<NamespaceId>  = Lazy::new(|| NamespaceId::new());

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
    fn default_target() -> StreamTarget {
        SubProgramId::called("flowbetween::canvas_renderer").into()
    }

    fn initialise(init_context: &impl SceneInitialisationContext) { 
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|msgs| msgs.map(|_: IdleNotification| CanvasRender::Idle))), (), StreamId::with_message_type::<IdleNotification>()).unwrap();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|msgs| msgs.map(|update| CanvasRender::Update(update)))), (), StreamId::with_message_type::<VectorCanvasUpdate>()).unwrap();

        init_context.add_subprogram(SubProgramId::called("flowbetween::canvas_renderer"), canvas_render_program, 50);
    }
}

///
/// Renders the canvas
///
pub async fn canvas_render_program(input: InputStream<CanvasRender>, context: SceneContext) {
    let our_program_id = context.current_program_id().unwrap();

    // Set to true when we've requested an idle event to complete a rendering operation
    let mut idle_requested = false;

    // General state: the frame to render and the transformation to apply to the canvas
    let mut frame_time  = FrameTime::ZERO;
    let mut transform   = Transform2D::identity();

    // List of layers that have been rendered and not invalidated
    let mut valid_layers = HashSet::<CanvasLayerId>::new();

    // Connect to our dependencies
    let mut idle_request    = context.send::<IdleRequest>(()).unwrap();
    let mut drawing         = context.send::<DrawingRequest>(()).unwrap();

    // When the program starts, set up a refresh
    if idle_request.send(IdleRequest::WhenIdle(our_program_id)).await.is_ok() {
        idle_requested = true;
    }

    // Run the main loop
    let mut input = input;
    while let Some(msg) = input.next().await {
        match msg {
            CanvasRender::Idle => {
                // There's no longer an idle request
                idle_requested = false;

                // TODO: render any layers in the document that are invalid
            }

            CanvasRender::Update(VectorCanvasUpdate::LayerChanged(layers)) => {
                // Invalidate these layers and request a redraw
                for layer in layers.into_iter() {
                    valid_layers.remove(&layer);
                }

                if !idle_requested && idle_request.send(IdleRequest::WhenIdle(our_program_id)).await.is_ok() {
                    idle_requested = true;
                }
            }

            CanvasRender::Update(VectorCanvasUpdate::ShapeChanged(_shapes)) => {
                // Nothing to do yet (if this was updated so we could tell when new shapes are added to the end of a layer, we could avoid redrawing the entire layer)
            }

            CanvasRender::Refresh => {
                // All layers become invalid and we request an idle event to force a redraw
                valid_layers.clear();

                if !idle_requested && idle_request.send(IdleRequest::WhenIdle(our_program_id)).await.is_ok() {
                    idle_requested = true;
                }
            }

            CanvasRender::SetTransform(new_transform) => {
                // Update the transform, invalidate the layers and request an idle event to redraw everything
                transform = new_transform;
                valid_layers.clear();

                if !idle_requested && idle_request.send(IdleRequest::WhenIdle(our_program_id)).await.is_ok() {
                    idle_requested = true;
                }
            }

            CanvasRender::SetFrame(new_frame_time) => {
                // Update the frame time, invalidate the layers and request an idle event to redraw everything
                frame_time = new_frame_time;
                valid_layers.clear();

                if !idle_requested && idle_request.send(IdleRequest::WhenIdle(our_program_id)).await.is_ok() {
                    idle_requested = true;
                }
            }
        }
    }
}