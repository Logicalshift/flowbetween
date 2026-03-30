use super::canvas_tool_type_ids::*;
use super::group_ids::*;
use super::tool::*;
use crate::scenery::ui::*;
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

use std::sync::*;

///
/// Current settings for the brush tool
///
#[derive(Clone)]
pub struct BrushToolState {
    /// Position where the tool is hovering
    hover_pos: Binding<Option<(f64, f64)>>,

    /// Whether or not the tool is selected
    tool_selected: Binding<bool>,

    /// Whether or not the mouse has entered the tool region
    mouse_over: Binding<bool>,

    /// Preview for the tool
    preview: Binding<Arc<Vec<Draw>>>,

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
        let mut preview = vec![];
        preview.new_path();
        preview.circle(0.0, 0.0, 10.0);
        preview.stroke_color(Color::Rgba(0.6, 0.6, 0.6, 1.0));
        preview.line_width_pixels(1.0);
        preview.stroke();

        Self {
            hover_pos:          bind(None),
            tool_selected:      bind(false),
            mouse_over:         bind(false),
            preview:            bind(Arc::new(preview)),
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
    let (hover_pos, tool_selected, mouse_over, preview, layer_transform) = {
        let data = data.lock().unwrap();

        (data.hover_pos.clone(), data.tool_selected.clone(), data.mouse_over.clone(), data.preview.clone(), data.layer_transform.clone())
    };

    let binding = computed(move || {
        // Get the properties
        let hover_pos       = hover_pos.get();
        let tool_selected   = tool_selected.get();
        let mouse_over      = mouse_over.get();
        let preview         = preview.get();

        let mut drawing = vec![];

        // Create the brush preview drawing
        drawing.push_state();

        // TODO: need to apply the canvas transform (but also need to apply the canvas transform to the coordinates we get from the tool focus)
        drawing.namespace(*CANVAS_OVERLAY_NAMESPACE);
        drawing.layer(LayerId(0));
        drawing.clear_layer();

        drawing.set_layer_transform(layer_transform.get());

        // Draw the preview if the mouse is over the canvas
        if let (Some(hover_pos), true, true) = (hover_pos, tool_selected, mouse_over) {
            drawing.transform(Transform2D::translate(hover_pos.0 as _, hover_pos.1 as _));
            drawing.extend(preview.iter().cloned());
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
