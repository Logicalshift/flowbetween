use super::canvas_tool_type_ids::*;
use super::group_ids::*;
use super::tool::*;
use crate::scenery::ui::*;
use crate::scenery::document::subprograms::*;

use flo_binding::*;
use flo_draw::*;
use flo_draw::canvas::*;
use flo_scene::*;

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
            hover_pos:      bind(None),
            tool_selected:  bind(false),
            mouse_over:     bind(false),
            preview:        bind(Arc::new(preview)),
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
    let behaviour = behaviour.with_canvas_program(|input, _context, data| async move {
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
