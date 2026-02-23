use super::layer_renderer::*;
use super::super::frame_time::*;
use super::super::layer::*;
use super::super::queries::*;
use super::super::vector_editor::*;

use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;
use flo_scene::*;
use flo_scene::programs::*;

use futures::prelude::*;
use futures::stream::{FuturesUnordered};
use serde::*;
use once_cell::sync::{Lazy};

use std::collections::*;
use std::sync::*;

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
    let mut idle_requested  = false;
    let mut need_redraw     = true;

    // General state: the frame to render and the transformation to apply to the canvas
    let mut frame_time  = FrameTime::ZERO;
    let mut transform   = Transform2D::identity();

    // List of layers that have been rendered and not invalidated
    let mut valid_layers        = HashSet::<CanvasLayerId>::new();
    let mut layer_map           = HashMap::new();
    let mut last_layer_order    = vec![];

    // Connect to our dependencies
    let mut idle_request    = context.send::<IdleRequest>(()).unwrap();
    let mut drawing_request = context.send::<DrawingRequest>(()).unwrap();

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

                if need_redraw {
                    need_redraw = false;

                    // Build up the drawing in a vec. We start by pushing the current state and switching to our namespace
                    let mut drawing = vec![];
                    drawing.push_state();
                    drawing.namespace(*CANVAS_NAMESPACE);
                    drawing.transform(transform);

                    // Get the layers and properties from the document outline
                    let mut properties  = vec![];
                    let mut layers      = HashMap::new();
                    let mut layer_order = vec![];

                    let mut outline     = query_vector_outline();
                    while let Some(outline) = outline.next().await {
                        match outline {
                            VectorResponse::Document(doc_properties)            => { properties = doc_properties; }
                            VectorResponse::Layer(layer_id, layer_properties)   => { layers.insert(layer_id, layer_properties); }
                            VectorResponse::LayerOrder(doc_layer_order)         => { layer_order = doc_layer_order; }
                            _                                                   => { }
                        }
                    }

                    // TODO: use the properties to render the document outline
                    let _ = properties;

                    // If the layer order has changed, then re-render everything
                    if layer_order != last_layer_order {
                        // Blank all of the layers in the map
                        for layer_id in layer_map.values() {
                            drawing.layer(*layer_id);
                            drawing.clear_layer();
                        }

                        // Clear out the layers
                        valid_layers.clear();
                        layer_map.clear();
                        last_layer_order = layer_order.clone();
                    }

                    // We render all layers that are in layers but not in valid_layers
                    let layers_to_render = layers.keys()
                        .filter(|layer_id| !valid_layers.contains(layer_id))
                        .copied()
                        .collect::<Vec<_>>();

                    // Query the layers that we're going to render, and bin them by layer
                    let mut layer_contents  = query_vector_layers(frame_time, layers_to_render.clone());
                    let mut layer_rendering = HashMap::new();
                    let mut current_layer   = None;

                    while let Some(layer_response) = layer_contents.next().await {
                        match layer_response {
                            VectorResponse::Layer(layer_id, _properties) => {
                                // Select the layer
                                current_layer = Some(layer_id);
                                layer_rendering.insert(layer_id, vec![]);
                            }

                            other => {
                                // Add to the current layer
                                if let Some(current_layer) = current_layer {
                                    layer_rendering.get_mut(&current_layer).unwrap().push(other);
                                }
                            }
                        }
                    }

                    // Assign layer IDs based on the ordering (layers have a layer above and below for previews/annotations)
                    let mut current_layer = 2;
                    for layer_id in layer_order.iter() {
                        layer_map.insert(*layer_id, LayerId(current_layer));

                        current_layer += 3;
                    }

                    // Render the layers to get their drawing instructions
                    let mut layer_instructions = layer_rendering
                        .into_iter()
                        .map(|(layer_id, layer_data)| render_layer(layer_data, frame_time, &context).map(move |drawing| (layer_id, drawing)))
                        .collect::<FuturesUnordered<_>>();

                    // Read the rendered layers and add to our drawing instructions
                    while let Some((canvas_layer_id, layer_drawing)) = layer_instructions.next().await {
                        let layer_id = layer_map.get(&canvas_layer_id).copied().unwrap();

                        drawing.layer(layer_id);
                        drawing.extend(layer_drawing);
                    }

                    // Finish the drawing
                    drawing.pop_state();

                    // Send to be renderered
                    drawing_request.send(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
                }
            }

            CanvasRender::Update(VectorCanvasUpdate::LayerChanged(layers)) => {
                // Invalidate these layers and request a redraw
                need_redraw = true;
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
                need_redraw = true;

                if !idle_requested && idle_request.send(IdleRequest::WhenIdle(our_program_id)).await.is_ok() {
                    idle_requested = true;
                }
            }

            CanvasRender::SetTransform(new_transform) => {
                // Update the transform, invalidate the layers and request an idle event to redraw everything
                transform = new_transform;
                valid_layers.clear();
                need_redraw = true;

                if !idle_requested && idle_request.send(IdleRequest::WhenIdle(our_program_id)).await.is_ok() {
                    idle_requested = true;
                }
            }

            CanvasRender::SetFrame(new_frame_time) => {
                // Update the frame time, invalidate the layers and request an idle event to redraw everything
                frame_time = new_frame_time;
                valid_layers.clear();
                need_redraw = true;

                if !idle_requested && idle_request.send(IdleRequest::WhenIdle(our_program_id)).await.is_ok() {
                    idle_requested = true;
                }
            }
        }
    }
}