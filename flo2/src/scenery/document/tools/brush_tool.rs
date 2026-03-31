use super::canvas_tool_type_ids::*;
use super::group_ids::*;
use super::tool::*;
use crate::scenery::ui::*;
use crate::scenery::document::brush::*;
use crate::scenery::document::canvas::*;
use crate::scenery::document::subprograms::*;

use flo_binding::*;
use flo_draw::*;
use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;
use flo_scene::*;
use flo_scene::programs::*;
use flo_scene_binding::*;

use futures::prelude::*;
use futures::channel::mpsc;
use futures::stream::FuturesUnordered;

use std::sync::*;

///
/// Current settings for the brush tool
///
#[derive(Clone)]
pub struct BrushToolState {
    /// The settings for the brush
    brush_settings: Binding<CoreBrushSettings>,

    /// Position where the tool is hovering
    hover_pos: Binding<Option<(f64, f64)>>,

    /// Whether or not the tool is selected
    tool_selected: Binding<bool>,

    /// Whether or not the mouse has entered the tool region
    mouse_over: Binding<bool>,

    /// True if the tool is currently drawing
    is_drawing: Binding<bool>,

    /// The transform that's applied to the binding layer
    layer_transform: Binding<Transform2D>,
}

impl ToolData for BrushToolState {
    fn initial_position(&self) -> (StreamTarget, (f64, f64)) {
        (subprogram_tool_dock_left().into(), (0.0, 1.0))
    }

    fn is_duplicate(&mut self, _is_duplicate: bool) { }

    fn selected(&mut self, is_selected: bool) {
        self.tool_selected.set(is_selected);
    }
}

impl Default for BrushToolState {
    fn default() -> Self {
        Self {
            brush_settings:     bind(CoreBrushSettings::default()),
            hover_pos:          bind(None),
            tool_selected:      bind(false),
            mouse_over:         bind(false),
            is_drawing:         bind(false),
            layer_transform:    bind(Transform2D::identity()),
        }
    }
}

///
/// Runs the brush tool program
///
pub async fn brush_tool_program(input: InputStream<ToolState>, context: SceneContext) {
    // Set up the behaviour
    let behaviour = ToolBehaviour::new("Brush", || vec![ BrushToolState::default() ]);

    // Ink icon
    let behaviour = behaviour.with_icon_svg(include_bytes!("../../../../../flo/svg/tools/ink.svg"));

    // The actual behaviour when focused on the canvas
    let behaviour = behaviour.with_canvas_program(|input, context, data| async move {
        let Some(our_program_id) = context.current_program_id() else { return; };

        // Tell SceneControl to run a child program that monitors the layer transform
        let transform_data = data.clone();
        context.send_message(SceneControl::start_child_program(SubProgramId::new(), our_program_id, move |input, context| brush_tool_canvas_state_tracker(input, context, transform_data), 1)).await.ok();

        // Tell SceneControl to run a child program that draws the brush preview
        let preview_data = data.clone();
        context.send_message(SceneControl::start_child_program(SubProgramId::new(), our_program_id, move |input, context| brush_tool_preview_program(input, context, preview_data), 1)).await.ok();

        // Monitor events
        let mut input = input;
        while let Some(msg) = input.next().await {
            match msg {
                FocusEvent::Pointer(FocusPointerEvent::Pointer(_, PointerAction::Enter, _, state)) => {
                    data.lock().unwrap().hover_pos.set(state.location_in_canvas);
                    data.lock().unwrap().mouse_over.set(true);
                }

                FocusEvent::Pointer(FocusPointerEvent::Pointer(_, PointerAction::Leave, _, _)) => {
                    data.lock().unwrap().hover_pos.set(None);
                    data.lock().unwrap().mouse_over.set(false);
                }

                FocusEvent::Pointer(FocusPointerEvent::Pointer(_, PointerAction::Move, _, state)) => {
                    data.lock().unwrap().hover_pos.set(state.location_in_canvas);
                }

                FocusEvent::Pointer(FocusPointerEvent::Pointer(_, PointerAction::ButtonDown, _, state)) => {
                    data.lock().unwrap().is_drawing.set(true);
                    brush_stroke(state.buttons[0], &mut input, &context, data.clone(), (*CANVAS_NAMESPACE, LayerId(1000))).await;
                    data.lock().unwrap().is_drawing.set(false);
                }

                _ => {}
            }
        }
    });

    // Run the tool program
    (tool_program(TOOL_BRUSH, TOOL_GROUP_CANVAS, behaviour))(input, context).await;
}

///
/// Subprogram that shows the brush preview
///
async fn brush_tool_preview_program(input: InputStream<BindingProgram>, context: SceneContext, data: Arc<Mutex<BrushToolState>>) {
    // Action is just to send a drawing request
    let action = BindingAction::new(|drawing: Arc<Vec<Draw>>, context| async move {
        context.send_message(DrawingRequest::Draw(drawing)).await.ok();
    });

    // Binding creates the drawing
    let (hover_pos, tool_selected, mouse_over, is_drawing, settings, layer_transform) = {
        let data = data.lock().unwrap();

        (data.hover_pos.clone(), data.tool_selected.clone(), data.mouse_over.clone(), data.is_drawing.clone(), data.brush_settings.clone(), data.layer_transform.clone())
    };

    let binding = computed(move || {
        // Get the properties
        let hover_pos       = hover_pos.get();
        let tool_selected   = tool_selected.get();
        let mouse_over      = mouse_over.get();
        let settings        = settings.get();
        let is_drawing      = is_drawing.get();
        let size            = 40.0;

        let mut drawing = vec![];

        // Create the brush preview drawing
        drawing.push_state();

        // TODO: need to apply the canvas transform (but also need to apply the canvas transform to the coordinates we get from the tool focus)
        drawing.namespace(*CANVAS_OVERLAY_NAMESPACE);
        drawing.layer(LayerId(0));
        drawing.clear_layer();

        drawing.set_layer_transform(layer_transform.get());

        // Draw the preview if the mouse is over the canvas
        if let (Some(hover_pos), true, true, false) = (hover_pos, tool_selected, mouse_over, is_drawing) {
            drawing.transform(Transform2D::translate(hover_pos.0 as _, hover_pos.1 as _));
            drawing.extend(settings.preview(PointerState { location_in_window: hover_pos, location_in_canvas: Some(hover_pos), buttons: vec![], pressure: None, tilt: None, rotation: None, flow_rate: None }, size));
        }

        drawing.pop_state();

        Arc::new(drawing)
    });

    // Run the binding program
    binding_program(input, context, binding, action).await;
}

///
/// Tracks the current layer transform for the canvas layers
///
async fn brush_tool_canvas_state_tracker(input: InputStream<CanvasRenderUpdate>, context: SceneContext, data: Arc<Mutex<BrushToolState>>) {
    let our_program_id = context.current_program_id().unwrap();

    // Request updates on the layer transform
    context.send_message(CanvasRender::Subscribe(our_program_id.into())).await.ok();

    // Monitor for updates
    let mut input = input;
    while let Some(msg) = input.next().await {
        match msg {
            CanvasRenderUpdate::LayerTransform(transform)   => { data.lock().unwrap().layer_transform.set(transform); },
            CanvasRenderUpdate::Layers(_layers)             => { },
        }
    }
}

///
/// Previews a brush stroke using this tool on the specified layer (returning the final brushstroke so it can be added to the canvas)
///
async fn brush_stroke(button_down: Button, input: &mut InputStream<FocusEvent>, context: &SceneContext, data: Arc<Mutex<BrushToolState>>, (namespace, layer): (NamespaceId, LayerId)) {
    use std::iter;

    // Query the brush status (TODO: also send the request to other tools)
    let core_settings = data.lock().unwrap().brush_settings.get();
    let core_response = core_settings.to_brush_responses();

    let layer_transform = data.lock().unwrap().layer_transform.get();

    // Generate the shapes we're going to preview
    let (send_points, recv_points)  = mpsc::channel(100);
    let preview_shapes              = create_shape_stream(recv_points, core_response.iter());

    // Create a 'task' that draws the shapes as they're generated by the shape stream
    let draw_preview_shapes = async move {
        let mut preview_shapes      = preview_shapes;
        let Ok(mut drawing_request) = context.send(()) else { return; };

        while let Some(shapes) = preview_shapes.next().await {
            let mut drawing = vec![];

            drawing.push_state();
            drawing.namespace(namespace);
            drawing.layer(layer);
            drawing.clear_layer();
            drawing.set_layer_transform(layer_transform);

            for shape in shapes {
                // Add a colour if the shape generated doesn't have one
                // TODO: temporary until we have an actual colour tool
                let mut shape = shape;
                if !shape.properties.iter().any(|(prop, _val)| prop == &*PROP_FILL_COLOR) {
                    let mut properties = Arc::unwrap_or_clone(shape.properties);

                    properties.push((*PROP_FILL_COLOR,      color_value_property(&Color::Rgba(0.0, 0.0, 0.0, 1.0))));
                    properties.push((*PROP_FILL_COLOR_TYPE, color_type_property(&Color::Rgba(0.0, 0.0, 0.0, 1.0))));

                    shape.properties = Arc::new(properties);
                }

                // Generate the drawing instructions for this shape
                let shape_drawing = render_shapes(iter::once(Arc::new(shape)), FrameTime::ZERO, context).await;

                // Add them to our drawing
                drawing.extend(shape_drawing.iter().flat_map(|item| item.iter().cloned()));
            }

            drawing.pop_state();

            // Send as a drawing requests
            drawing_request.send(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
        }
    };

    // Create another task that reads from the input and generates brush points untils the mouse button is released
    let track_events = async move {
        let mut send_points = send_points;

        // Process mouse events until we're done
        while let Some(evt) = input.next().await {
            match evt {
                FocusEvent::Pointer(FocusPointerEvent::Pointer(_, PointerAction::Move, _, state)) => { 
                    // Generate a brush point
                    let brush_point = BrushPoint::from(state);

                    // Send to the drawing (other task should pick it up)
                    let Ok(_) = send_points.send(brush_point).await else { return; };
                }

                FocusEvent::Pointer(FocusPointerEvent::Pointer(_, PointerAction::ButtonUp, _, _state)) => {
                    // TODO: check that the button being released is 'button_down'
                    break;
                }

                _ => { }
            }
        }
    };

    // Combine both into a FuturesUnordered
    let mut tasks = FuturesUnordered::new();
    tasks.push(draw_preview_shapes.boxed());
    tasks.push(track_events.boxed());

    // Stop when either tasks finishes (discarding the other)
    tasks.next().await;
    drop(tasks);

    // Clear up the preview layer
    let mut clear_preview = vec![];

    clear_preview.push_state();
    clear_preview.namespace(namespace);
    clear_preview.layer(layer);
    clear_preview.clear_layer();
    clear_preview.pop_state();

    context.send_message(DrawingRequest::Draw(Arc::new(clear_preview))).await.ok();

    // TODO: the last set of shapes generated is the result of this function
}
