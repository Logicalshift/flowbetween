use super::constants::*;
use super::adjust_tool::*;

use super::adjust_edge::*;
use super::adjust_state::*;

use super::super::select::*;

use crate::tools::*;

use flo_ui::*;
use flo_stream::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;

use std::f64;
use std::sync::*;

impl Adjust {
    ///
    /// Performs a drag on an edge
    ///
    pub (super) async fn drag_edge<Anim: 'static+EditableAnimation>(state: &mut AdjustToolState<Anim>, selected_edge: AdjustEdgePoint, initial_event: Painting) {
        // Fetch the element being transformed and their properties
        let when                = state.flo_model.timeline().current_time.get();
        let frame               = state.flo_model.frame().frame.get();
        let frame               = if let Some(frame) = frame { frame } else { return; };

        let element             = frame.element_with_id(selected_edge.start_point.owner);
        let element             = if let Some(element) = element { element } else { return; };
        let element_properties  = frame.apply_properties_for_element(&element, Arc::new(VectorProperties::default()));

        // Decompose the initial position
        let (x1, y1) = initial_event.location;

        // Read the inputs for the drag
        while let Some(event) = state.input.next().await {
            match event {
                ToolInput::Paint(paint_event) => {
                    if paint_event.pointer_id != initial_event.pointer_id {
                        // Ignore events from other devices
                        continue;
                    }

                    match paint_event.action {
                        PaintAction::Continue |
                        PaintAction::Prediction => {
                            // Drag the control points and redraw the preview
                            let (x2, y2)                = paint_event.location;
                            let (dx, dy)                = (x2-x1, y2-y1);

                            let adjusted_control_points = state.adjusted_control_points_for_curve_drag(&selected_edge, dx as f64, dy as f64);

                            // Create the adjusted element
                            let adjusted_element        = element.with_adjusted_control_points(adjusted_control_points, &*element_properties);

                            // Draw the updated elements
                            let mut preview = vec![Draw::Layer(LAYER_SELECTION), Draw::ClearLayer];
                            preview.extend(Select::highlight_for_selection(&adjusted_element, &*element_properties, true).0);
                            preview.extend(vec![Draw::Layer(LAYER_PREVIEW), Draw::ClearLayer]);

                            state.actions.send_actions(vec![ToolAction::Overlay(OverlayAction::Draw(preview))]);
                        }

                        PaintAction::Finish => {
                            // Commit the drag to the drawing
                            let (x2, y2)                = paint_event.location;
                            let (dx, dy)                = (x2-x1, y2-y1);

                            // Compile the edits
                            let adjusted_control_points = state.adjusted_control_points_for_curve_drag(&selected_edge, dx as f64, dy as f64);
                            let edits                   = vec![AnimationEdit::Element(vec![element.id()], ElementEdit::SetControlPoints(adjusted_control_points, when))];

                            // Send to the animation (invalidating the canvas will redraw the selection to its final value)
                            state.flo_model.edit().publish(Arc::new(edits)).await;
                            state.flo_model.timeline().invalidate_canvas();

                            // Drag is finished
                            return;
                        }

                        PaintAction::Start  |
                        PaintAction::Cancel => {
                            // Reset the preview
                            let mut preview = vec![Draw::Layer(LAYER_SELECTION), Draw::ClearLayer];
                            preview.extend(Self::drawing_for_selection_preview(&*state.flo_model));
                            preview.extend(vec![Draw::Layer(LAYER_PREVIEW), Draw::ClearLayer]);
                            preview.extend(Self::drawing_for_control_points(&*state.control_points.get(), &state.selected_control_points.get()));

                            state.actions.send_actions(vec![ToolAction::Overlay(OverlayAction::Draw(preview))]);

                            // Abort the drag
                            return;
                        }
                    }
                }

                _ => { }
            }
        }

        // Input stream ended prematurely
    }
}
