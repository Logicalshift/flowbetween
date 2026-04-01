use super::constants::*;
use super::adjust_tool::*;

use super::adjust_control_point::*;

use super::super::select::*;

use crate::tools::*;
use crate::model::*;
use crate::style::*;

use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;

use std::f32;
use std::sync::*;
use std::collections::{HashSet};

impl Adjust {
    ///
    /// Writes out a control point sprite for a bezier point
    ///
    pub (super) fn declare_bezier_point_sprite(sprite_id: SpriteId, selected: bool) -> Vec<Draw> {
        let mut draw            = vec![];
        const RADIUS: f32       = 3.0;
        const NUM_SIDES: u32    = 12;

        // Draw to the bezier sprite
        draw.sprite(sprite_id);
        draw.clear_sprite();

        // Render as a polygon rather than a circle to reduce the number of triangles we'll need to render for this sprite
        draw.new_path();
        draw.move_to(0.0, RADIUS+1.0);
        for point in 1..NUM_SIDES {
            let angle = (2.0*f32::consts::PI)*((point as f32)/(NUM_SIDES as f32));
            draw.line_to(angle.sin()*(RADIUS+1.0), angle.cos()*(RADIUS+1.0));
        }
        draw.close_path();
        if selected {
            draw.fill_color(CP_BEZIER_SELECTED_OUTLINE);
        } else {
            draw.fill_color(CP_BEZIER_OUTLINE);
        }
        draw.fill();

        draw.new_path();
        draw.move_to(0.0, RADIUS);
        for point in 1..NUM_SIDES {
            let angle = (2.0*f32::consts::PI)*((point as f32)/(NUM_SIDES as f32));
            draw.line_to(angle.sin()*RADIUS, angle.cos()*RADIUS);
        }
        draw.close_path();
        if selected {
            draw.fill_color(CP_BEZIER_SELECTED);
        } else {
            draw.fill_color(CP_BEZIER);
        }
        draw.fill();

        draw
    }

    ///
    /// Writes out a control point sprite for a bezier point
    ///
    pub (super) fn declare_control_point_sprite(sprite_id: SpriteId) -> Vec<Draw> {
        let mut draw            = vec![];
        const RADIUS: f32       = 2.0;

        // Draw to the bezier sprite
        draw.sprite(sprite_id);
        draw.clear_sprite();

        // Render as a polygon rather than a circle to reduce the number of triangles we'll need to render for this sprite
        draw.new_path();
        draw.move_to(-(RADIUS+1.0), RADIUS+1.0);
        draw.line_to(RADIUS+1.0, RADIUS+1.0);
        draw.line_to(RADIUS+1.0, -(RADIUS+1.0));
        draw.line_to(-(RADIUS+1.0), -(RADIUS+1.0));
        draw.close_path();
        draw.fill_color(CP_BEZIER_OUTLINE);
        draw.fill();

        draw.new_path();
        draw.move_to(-RADIUS, RADIUS);
        draw.line_to(RADIUS, RADIUS);
        draw.line_to(RADIUS, -RADIUS);
        draw.line_to(-RADIUS, -RADIUS);
        draw.close_path();
        draw.fill_color(CP_BEZIER_CP);
        draw.fill();

        draw
    }

    ///
    /// Generates the selection preview drawing actions from a model
    ///
    pub (super) fn drawing_for_selection_preview<Anim: 'static+EditableAnimation>(flo_model: &FloModel<Anim>) -> Vec<Draw> {
        // Determine the selected elements from the model
        let current_frame           = flo_model.frame().frame.get();
        let selected_elements       = flo_model.selection().selected_elements.get();

        // Fetch the elements from the frame and determine how to draw the highlight for them
        let mut selection_drawing   = vec![];

        if let Some(current_frame) = current_frame.as_ref() {
            for selected_id in selected_elements.iter() {
                let element = current_frame.element_with_id(*selected_id);

                if let Some(element) = element {
                    // Update the properties according to this element
                    let properties  = current_frame.apply_properties_for_element(&element, Arc::new(VectorProperties::default()));

                    // Draw the highlight for it
                    let (drawing, _bounds) = Select::highlight_for_selection(&element, &properties, false);
                    selection_drawing.extend(drawing);
                }
            }
        }

        selection_drawing
    }

    ///
    /// Renders the control points (without adjustment handles) for the current selection
    ///
    pub (super) fn drawing_for_control_points(control_points: &Vec<AdjustControlPoint>, selected_control_points: &HashSet<AdjustControlPointId>) -> Vec<Draw> {
        let mut drawing = vec![];

        // Draw the control lines for the selected control point, if there is only one
        if selected_control_points.len() == 1 {
            let selected_control_point = selected_control_points.iter().nth(0).unwrap();

            for cp_index in 0..control_points.len() {
                let cp = &control_points[cp_index];
                let (x1, y1) = cp.control_point.position();

                if cp.owner == selected_control_point.owner && cp.index == selected_control_point.index {
                    // Draw the control points for this CP (preceding and following point)
                    drawing.line_width_pixels(1.0);
                    drawing.stroke_color(CP_LINES);

                    if cp_index > 0 {
                        let preceding = &control_points[cp_index-1];
                        if preceding.control_point.is_control_point() {
                            let (x2, y2) = preceding.control_point.position();

                            drawing.new_path();
                            drawing.move_to(x1 as f32, y1 as f32);
                            drawing.line_to(x2 as f32, y2 as f32);
                            drawing.stroke();
                        }
                    }

                    if cp_index < control_points.len()-1 {
                        let following = &control_points[cp_index+1];
                        if following.control_point.is_control_point() {
                            let (x2, y2) = following.control_point.position();

                            drawing.new_path();
                            drawing.move_to(x1 as f32, y1 as f32);
                            drawing.line_to(x2 as f32, y2 as f32);
                            drawing.stroke();
                        }
                    }
                }
            }
        }

        // Draw the main control points
        for cp in control_points.iter().filter(|cp| !cp.control_point.is_control_point()) {
            // Draw a control point sprite
            let (x, y) = cp.control_point.position();

            drawing.sprite_transform(SpriteTransform::Identity);
            drawing.sprite_transform(SpriteTransform::Translate(x as f32, y as f32));

            if selected_control_points.contains(&AdjustControlPointId { owner: cp.owner, index: cp.index }) {
                drawing.draw_sprite(SPRITE_SELECTED_BEZIER_POINT);
            } else {
                drawing.draw_sprite(SPRITE_BEZIER_POINT);
            }
        }

        // Draw the control points for the selected control point, if there is only one
        if selected_control_points.len() == 1 {
            let selected_control_point = selected_control_points.iter().nth(0).unwrap();

            for cp_index in 0..control_points.len() {
                let cp = &control_points[cp_index];

                if cp.owner == selected_control_point.owner && cp.index == selected_control_point.index {
                    // Draw the control points for this CP (preceding and following point)
                    if cp_index > 0 {
                        let preceding = &control_points[cp_index-1];
                        if preceding.control_point.is_control_point() {
                            let (x, y) = preceding.control_point.position();

                            drawing.sprite_transform(SpriteTransform::Identity);
                            drawing.sprite_transform(SpriteTransform::Translate(x as f32, y as f32));
                            drawing.draw_sprite(SPRITE_BEZIER_CONTROL_POINT);
                        }
                    }

                    if cp_index < control_points.len()-1 {
                        let following = &control_points[cp_index+1];
                        if following.control_point.is_control_point() {
                            let (x, y) = following.control_point.position();

                            drawing.sprite_transform(SpriteTransform::Identity);
                            drawing.sprite_transform(SpriteTransform::Translate(x as f32, y as f32));
                            drawing.draw_sprite(SPRITE_BEZIER_CONTROL_POINT);
                        }
                    }
                }
            }
        }

        drawing
    }

    ///
    /// Tracks the selection path and renders the control points and selection preview
    ///
    pub (super) async fn render_selection_path<Anim: 'static+EditableAnimation>(actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>, control_points: BindRef<Arc<Vec<AdjustControlPoint>>>, selected_control_points: BindRef<HashSet<AdjustControlPointId>>) {
        // Create a binding that tracks the rendering actions for the current selection
        let model               = flo_model.clone();
        let selection_preview   = computed(move || Self::drawing_for_selection_preview(&*model));
        let cp_preview          = computed(move || Self::drawing_for_control_points(&*control_points.get(), &selected_control_points.get()));

        // Combine the two previews whenever the selection changes
        let preview             = computed(move || {
            let selection_preview       = selection_preview.get();
            let cp_preview              = cp_preview.get();

            let mut preview         = vec![Draw::Layer(LAYER_SELECTION), Draw::ClearLayer];
            preview.extend(selection_preview);
            preview.extend(vec![Draw::Layer(LAYER_PREVIEW), Draw::ClearLayer]);
            preview.extend(cp_preview);

            preview
        });

        // Draw the preview whenever it changes
        let mut preview = follow(preview);

        while let Some(new_preview) = preview.next().await {
            actions.send_actions(vec![
                ToolAction::Overlay(OverlayAction::Draw(new_preview))
            ])
        }
    }
}
