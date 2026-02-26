use super::layer_renderer::*;
use super::super::document_properties::*;
use super::super::frame_time::*;
use super::super::layer::*;
use super::super::property::*;
use super::super::queries::*;
use super::super::vector_editor::*;
use crate::scenery::ui::*;

use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;
use flo_draw::events::*;
use flo_scene::*;
use flo_scene::programs::*;
use flo_curves::*;

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

    /// Focus message (we're really only interested in resizing events)
    Focus(FocusEvent),

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
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|msgs| msgs.map(|focus| CanvasRender::Focus(focus)))), (), StreamId::with_message_type::<FocusEvent>()).unwrap();

        init_context.add_subprogram(SubProgramId::called("flowbetween::canvas_renderer"), canvas_render_program, 50);
    }
}

///
/// Calculates the layer transform for the canvas
///
fn calculate_layer_transform(transform: Transform2D, canvas_size: (f64, f64), window_size: (f64, f64)) -> Transform2D {
    // Move the center of the canvas to 0,0
    let canvas_center_x = (canvas_size.0 / 2.0).floor();
    let canvas_center_y = (canvas_size.1 / 2.0).floor();

    let move_canvas_center = Transform2D::translate(-canvas_center_x as _, -canvas_center_y as _);

    // Apply the transformation
    let with_transform      = transform * move_canvas_center;

    // Move back to the center of the window
    let window_center_x = (window_size.0/2.0).ceil();
    let window_center_y = (window_size.1/2.0).ceil();

    let center_in_window = Transform2D::translate(window_center_x as _, window_center_y as _) * with_transform;

    center_in_window
}

///
/// Renders the canvas
///
pub async fn canvas_render_program(input: InputStream<CanvasRender>, context: SceneContext) {
    let our_program_id = context.current_program_id().unwrap();

    // Set to true when we've requested an idle event to complete a rendering operation
    let mut idle_requested      = false;
    let mut need_redraw         = true;
    let mut update_transform    = true;

    // General state: the frame to render and the transformation to apply to the canvas
    let mut frame_time      = FrameTime::ZERO;
    let mut transform       = Transform2D::identity();
    let mut layer_transform = Transform2D::identity();
    let mut size_w          = 1920.0;
    let mut size_h          = 1080.0;
    let mut doc_w           = 0.0;
    let mut doc_h           = 0.0;
    let mut scale           = 1.0;

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

    // Claim an empty region in the focus program (we want resizing messages, but don't actually have any need of mouse events)
    context.send_message(Focus::ClaimRegion { program: our_program_id, region: vec![], z_index: 0 }).await.unwrap();

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
                        drawing.clear_layer();
                        drawing.set_layer_transform(layer_transform);
                        drawing.extend(layer_drawing);
                    }

                    // Update the document size if necessary
                    if let Some(doc_size) = DocumentSize::from_properties(properties.iter()) {
                        if doc_size.width != doc_w || doc_size.height != doc_h {
                            // Update the document size
                            doc_w = doc_size.width;
                            doc_h = doc_size.height;

                            // The transform for all the layers will change if the document size changes
                            update_transform = true;
                        }
                    }

                    // Finish the drawing
                    drawing.pop_state();

                    // Send to be renderered
                    drawing_request.send(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
                }
            }

            CanvasRender::Focus(evt) => {
                match evt {
                    FocusEvent::Event(_, DrawEvent::Resize(new_w, new_h)) => {
                        if (new_w/scale) != size_w || (new_h/scale) != size_h {
                            // Update the size
                            size_w = new_w/scale;
                            size_h = new_h/scale;

                            // Schedule a redraw
                            need_redraw         = true;
                            update_transform    = true;

                            if !idle_requested && idle_request.send(IdleRequest::WhenIdle(our_program_id)).await.is_ok() {
                                idle_requested = true;
                            }
                        }
                    }

                    FocusEvent::Event(_, DrawEvent::Scale(new_scale)) => {
                        if new_scale != scale {
                            // Update the size
                            size_w = (size_w*scale)/new_scale;
                            size_h = (size_h*scale)/new_scale;

                            // Update the scale
                            scale = new_scale;

                            // TODO: if we could move the layers without needing to completely regenerate them we wouldn't need to invalidate them here
                            valid_layers.clear();

                            // Schedule a redraw
                            need_redraw         = true;
                            update_transform    = true;

                            if !idle_requested && idle_request.send(IdleRequest::WhenIdle(our_program_id)).await.is_ok() {
                                idle_requested = true;
                            }
                        }
                    }

                    _ => { }
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
                // Update the transform. We use layer transforms to move the rendering around
                transform = new_transform;

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

        // Update the transformation for all the layers if needed
        if update_transform {
            let mut drawing = vec![];
            drawing.push_state();
            drawing.namespace(*CANVAS_NAMESPACE);

            layer_transform = calculate_layer_transform(transform, (doc_w, doc_h), (size_w, size_h));

            for (_, layer_id) in layer_map.iter() {
                drawing.layer(*layer_id);
                drawing.set_layer_transform(layer_transform);
            }

            // Figure out the scale of a pixel after the transform is applied
            let transform_scale = layer_transform.transform_coord(UiPoint(0.0, 0.0)).distance_to(&layer_transform.transform_coord(UiPoint(0.0, 1.0)));
            let transform_scale = 1.0 / transform_scale;

            // Redraw the document frame on layer 0
            drawing.layer(LayerId(0));
            drawing.clear_layer();
            drawing.set_layer_transform(layer_transform);

            drawing.fill_color(Color::Rgba(0.4, 0.4, 0.4, 1.0));
            drawing.new_path();
            drawing.rect((4.0*transform_scale) as _, (doc_h-4.0*transform_scale) as _, (doc_w-4.0*transform_scale) as _, (doc_h+4.0*transform_scale) as _);
            drawing.fill();

            drawing.line_width((1.0 * transform_scale) as _);
            drawing.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
            drawing.line_join(LineJoin::Miter);
            drawing.new_dash_pattern();
            drawing.fill_color(Color::Rgba(1.0, 1.0, 1.0, 1.0));

            drawing.new_path();
            drawing.rect(-0.5, -0.5, (doc_w+1.0) as _, (doc_h+1.0) as _);
            drawing.fill();
            drawing.stroke();

            // Transform has been updated: 
            update_transform = false;

            drawing.pop_state();
            drawing_request.send(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn layer_transform_centers_in_window() {
        // By default the canvas should be centered in the window
        let canvas_size = (100.0, 200.0);
        let window_size = (500.0, 700.0);
        let transform   = calculate_layer_transform(Transform2D::identity(), canvas_size, window_size);

        // Point at the center of the canvas should end up in the center of the window
        let canvas_center = (canvas_size.0/2.0, canvas_size.1/2.0);
        let window_center = (window_size.0/2.0, window_size.1/2.0);

        let new_center = transform.transform_point(canvas_center.0 as _, canvas_center.1 as _);

        assert!((new_center.0 as f64 - window_center.0).abs() < 0.0001, "{:?} != {:?}", new_center, window_center);
        assert!((new_center.1 as f64 - window_center.1).abs() < 0.0001, "{:?} != {:?}", new_center, window_center);
    }

    #[test]
    pub fn scales_around_center() {
        // Just scaling the canvas should leave the center point in the window as the center point in the canvas
        let canvas_size = (100.0, 200.0);
        let window_size = (500.0, 700.0);
        let transform   = calculate_layer_transform(Transform2D::scale(2.0, 2.0), canvas_size, window_size);

        // Point at the center of the canvas should end up in the center of the window (even with the scaling)
        let canvas_center = (canvas_size.0/2.0, canvas_size.1/2.0);
        let window_center = (window_size.0/2.0, window_size.1/2.0);

        let new_center = transform.transform_point(canvas_center.0 as _, canvas_center.1 as _);

        assert!((new_center.0 as f64 - window_center.0).abs() < 0.0001, "{:?} != {:?}", new_center, window_center);
        assert!((new_center.1 as f64 - window_center.1).abs() < 0.0001, "{:?} != {:?}", new_center, window_center);
    }

    #[test]
    pub fn translates_center() {
        // If we translate the canvas by 20, 30 then the centered point in the window should be -20, -30 from the original center
        let canvas_size = (100.0, 200.0);
        let window_size = (500.0, 700.0);
        let transform   = calculate_layer_transform(Transform2D::translate(20.0, 30.0), canvas_size, window_size);

        // Point at the center of the canvas should end up in the center of the window
        let canvas_center = (canvas_size.0/2.0, canvas_size.1/2.0);
        let window_center = (window_size.0/2.0, window_size.1/2.0);

        let new_center = transform.transform_point(canvas_center.0 as f32 - 20.0, canvas_center.1 as f32 - 30.0);

        assert!((new_center.0 as f64 - window_center.0).abs() < 0.0001, "{:?} != {:?}", new_center, window_center);
        assert!((new_center.1 as f64 - window_center.1).abs() < 0.0001, "{:?} != {:?}", new_center, window_center);
    }

    #[test]
    pub fn center_preserved_after_scaling() {
        // If we translate then scale the canvas, the centered point should be the same as if the canvas hadn't been scaled
        // With flo_canvas's transforms, 'A*B' = 'B then A' so the translation is last here
        let canvas_size = (100.0, 200.0);
        let window_size = (500.0, 700.0);
        let transform   = calculate_layer_transform(Transform2D::scale(2.0, 2.0) * Transform2D::translate(20.0, 30.0), canvas_size, window_size);

        // Point at the center of the canvas should end up in the center of the window
        let canvas_center = (canvas_size.0/2.0, canvas_size.1/2.0);
        let window_center = (window_size.0/2.0, window_size.1/2.0);

        let new_center = transform.transform_point(canvas_center.0 as f32 - 20.0, canvas_center.1 as f32 - 30.0);

        assert!((new_center.0 as f64 - window_center.0).abs() < 0.0001, "{:?} != {:?}", new_center, window_center);
        assert!((new_center.1 as f64 - window_center.1).abs() < 0.0001, "{:?} != {:?}", new_center, window_center);
    }
}
