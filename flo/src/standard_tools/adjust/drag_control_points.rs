use super::constants::*;
use super::adjust_tool::*;
use super::adjust_state::*;
use super::adjust_control_point::*;

use super::super::select::*;

use crate::tools::*;

use flo_ui::*;
use flo_stream::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;

use std::sync::*;
use std::collections::{HashSet, HashMap};

impl Adjust {
    ///
    /// Returns the list of modified control points for an element
    ///
    fn adjusted_control_points(offset: (f32, f32), element_id: ElementId, original_control_points: &Vec<ControlPoint>, selected_control_points: &HashSet<AdjustControlPointId>) -> Vec<(f32, f32)> {
        let (dx, dy) = offset;

        // Transform any control point that has changed
        let mut new_positions   = vec![];
        for cp_index in 0..original_control_points.len() {
            let cp_id = AdjustControlPointId {
                owner: element_id,
                index: cp_index
            };

            let cp          = &original_control_points[cp_index];
            let (cpx, cpy)  = cp.position();
            let (cpx, cpy)  = (cpx as f32, cpy as f32);
            if selected_control_points.contains(&cp_id) {
                // Transform this control point
                new_positions.push((cpx + dx, cpy + dy));
            } else if cp.is_control_point() 
                && (selected_control_points.contains(&AdjustControlPointId { owner: element_id, index: cp_index+1 }) && !original_control_points[cp_index+1].is_control_point()
                    || (cp_index > 0 && selected_control_points.contains(&AdjustControlPointId { owner: element_id, index: cp_index-1 }) && !original_control_points[cp_index-1].is_control_point())) {
                // Not selected, but the following CP is and this is a control point, so it transforms alongside it
                new_positions.push((cpx + dx, cpy + dy));
            } else {
                // Leave the control point alone
                new_positions.push((cpx, cpy));
            }
        }

        new_positions
    }

    ///
    /// Performs a drag on a set of control points
    ///
    pub (super) async fn drag_control_points<Anim: 'static+EditableAnimation>(state: &mut AdjustToolState<Anim>, selected_control_points: &HashSet<AdjustControlPointId>, initial_event: Painting) {
        // Fetch the elements being transformed and their properties
        let mut elements    = HashMap::new();
        let when            = state.flo_model.timeline().current_time.get();
        let frame           = state.flo_model.frame().frame.get();
        let frame           = if let Some(frame) = frame { frame } else { return; };

        for cp in selected_control_points.iter() {
            if !elements.contains_key(&cp.owner) {
                // Fetch the element (ignore elements that don't exist in the frame)
                let cp_element      = frame.element_with_id(cp.owner);
                let cp_element      = if let Some(cp_element) = cp_element { cp_element } else { continue; };

                // Calculate the properties for this element
                let cp_properties   = frame.apply_properties_for_element(&cp_element, Arc::new(VectorProperties::default()));

                // Store in the list of elements
                elements.insert(cp.owner, (cp_element, cp_properties));
            }
        }

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
                            let (x2, y2) = paint_event.location;
                            let (dx, dy) = (x2-x1, y2-y1);

                            // Move the control points of each element in the selection
                            let mut transformed_elements        = vec![];
                            let mut transformed_control_points  = vec![];
                            for (element, element_properties) in elements.values() {
                                // Fetch the control points for this element
                                let control_points  = element.control_points(&*element_properties);
                                let element_id      = element.id();

                                // Transform any control point that has changed
                                let new_positions   = Self::adjusted_control_points((dx, dy), element_id, &control_points, selected_control_points);

                                // Create an updated element with the new control points
                                let transformed_element = element.with_adjusted_control_points(new_positions, &*element_properties);

                                // Store the updated control points for the new element by re-reading them
                                transformed_control_points.extend(transformed_element.control_points(&*element_properties)
                                    .into_iter()
                                    .enumerate()
                                    .map(|(cp_index, cp)| AdjustControlPoint {
                                        owner:          element_id,
                                        index:          cp_index,
                                        control_point:  cp
                                    }));

                                // Remember the updated element to render it later
                                transformed_elements.push((transformed_element, Arc::clone(element_properties)));
                            }

                            // Draw the updated elements
                            let mut preview = vec![Draw::Layer(LAYER_SELECTION), Draw::ClearLayer];
                            preview.extend(transformed_elements.iter().flat_map(|(element, properties)| Select::highlight_for_selection(element, properties, true).0));
                            preview.extend(vec![Draw::Layer(LAYER_PREVIEW), Draw::ClearLayer]);
                            preview.extend(Self::drawing_for_control_points(&transformed_control_points, &state.selected_control_points.get()));

                            state.actions.send_actions(vec![ToolAction::Overlay(OverlayAction::Draw(preview))]);
                        }

                        PaintAction::Finish => {
                            // Commit the drag to the drawing
                            let (x2, y2) = paint_event.location;
                            let (dx, dy) = (x2-x1, y2-y1);

                            // Compile the edits
                            let mut edits = vec![];
                            for (element, element_properties) in elements.values() {
                                // Fetch the control points for this element
                                let control_points  = element.control_points(&*element_properties);
                                let element_id      = element.id();

                                // Transform any control point that has changed
                                let new_positions   = Self::adjusted_control_points((dx, dy), element_id, &control_points, selected_control_points);
                                edits.push(AnimationEdit::Element(vec![element_id], ElementEdit::SetControlPoints(new_positions, when)));
                            }

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
