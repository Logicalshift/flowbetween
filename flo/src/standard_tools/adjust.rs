use super::select::*;

use crate::menu::*;
use crate::tools::*;
use crate::model::*;
use crate::style::*;

use flo_ui::*;
use flo_stream::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;
use futures::stream::{BoxStream};

use std::f32;
use std::f64;
use std::iter;
use std::sync::*;
use std::collections::{HashSet, HashMap};

/// Layer where the current selection is drawn
const LAYER_SELECTION: u32 = 0;

/// Layer where the control points and the preview of the region the user is dragging is drawn
const LAYER_PREVIEW: u32 = 1;

///
/// A control point for the adjust tool
///
#[derive(Clone, Debug, PartialEq)]
struct AdjustControlPoint {
    owner:          ElementId,
    index:          usize,
    control_point:  ControlPoint
}

///
/// Identifier for a control point
///
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct AdjustControlPointId {
    owner:  ElementId,
    index:  usize
}

impl From<&AdjustControlPoint> for AdjustControlPointId {
    fn from(cp: &AdjustControlPoint) -> AdjustControlPointId {
        AdjustControlPointId {
            owner: cp.owner,
            index: cp.index
        }
    }
}

///
/// The current state of the input handler for the adjust tool
///
struct AdjustToolState<Anim: 'static+EditableAnimation> {
    input:                      ToolInputStream<()>, 
    actions:                    ToolActionPublisher<()>,
    flo_model:                  Arc<FloModel<Anim>>,
    control_points:             BindRef<Arc<Vec<AdjustControlPoint>>>,
    selected_control_points:    Binding<HashSet<AdjustControlPointId>>,
}

///
/// The model for the Adjust tool
///
pub struct AdjustModel {
    /// The future runs the adjust tool
    future: Mutex<ToolFuture>,
}

///
/// The Adjust tool, which alters control points and lines
///
pub struct Adjust { }

impl<Anim: 'static+EditableAnimation> AdjustToolState<Anim> {
    ///
    /// Finds the control point nearest to the specified position
    ///
    pub fn control_point_at_position(&self, x: f64, y: f64) -> Option<AdjustControlPointId> {
        const MIN_DISTANCE: f64         = 4.0;

        let mut found_distance          = 1000.0;
        let mut found_control_point     = None;

        for cp in self.control_points.get().iter().rev() {
            if cp.control_point.is_control_point() { continue; }

            let (cp_x, cp_y)        = cp.control_point.position();
            let x_diff              = cp_x - x;
            let y_diff              = cp_y - y;
            let distance_squared    = (x_diff*x_diff) + (y_diff)*(y_diff);

            if distance_squared < found_distance && distance_squared < MIN_DISTANCE*MIN_DISTANCE {
                found_distance      = distance_squared;
                found_control_point = Some(AdjustControlPointId { owner: cp.owner, index: cp.index });
            }
        }

        found_control_point
    }

    ///
    /// The control points to drag at the specified position, if they're different to the selection
    ///
    pub fn drag_control_points(&self, x: f64, y: f64) -> Option<HashSet<AdjustControlPointId>> {
        const MAX_DISTANCE: f64         = 8.0;
        let selected_control_points     = self.selected_control_points.get();

        if selected_control_points.len() == 1 {
            // If only one control point is selected, the user might drag the handles on either side
            let control_points  = self.control_points.get();
            let center_cp       = selected_control_points.iter().nth(0).cloned().unwrap();

            // Search for the center CP
            for cp_index in 0..control_points.len() {
                let cp = &control_points[cp_index];

                if cp.owner == center_cp.owner && cp.index == center_cp.index {
                    if cp.control_point.is_control_point() {
                        // Doesn't have handles
                        break;
                    }

                    // The left and right points might be the handles for this item
                    if cp_index > 0 && control_points[cp_index-1].control_point.is_control_point() {
                        // This CP is being dragged if it's within MAX_DISTANCE of the click
                        let (x2, y2) = control_points[cp_index-1].control_point.position();
                        let (dx, dy) = ((x-x2), (y-y2));

                        if (dx*dx) + (dy*dy) <= MAX_DISTANCE { return Some(iter::once((&control_points[cp_index-1]).into()).collect()); }
                    }

                    if cp_index < control_points.len()-1 && control_points[cp_index+1].control_point.is_control_point() {
                        // This CP is being dragged if it's within MAX_DISTANCE of the click
                        let (x2, y2) = control_points[cp_index+1].control_point.position();
                        let (dx, dy) = ((x-x2), (y-y2));

                        if (dx*dx) + (dy*dy) <= MAX_DISTANCE { return Some(iter::once((&control_points[cp_index+1]).into()).collect()); }
                    }

                    // Found the control point: don't look at the others
                    break;
                }
            }

            None
        } else {
            // Drag all of the selected control points, or select new control points if more than one is selected (or if 0 are selected)
            None
        }
    }

    ///
    /// Finds the element at the specified position
    ///
    pub fn element_at_position(&self, x: f64, y: f64) -> Option<ElementId> {
        // Find the elements at this point
        let frame       = self.flo_model.frame();
        let elements    = frame.elements_at_point((x as f32, y as f32));

        // Search for an element to select
        let mut selected_element = None;
        for elem in elements {
            match elem {
                ElementMatch::InsidePath(element) => {
                    selected_element = Some(element);
                    break;
                }

                ElementMatch::OnlyInBounds(element) => {
                    if selected_element.is_none() { selected_element = Some(element); }
                }
            }
        }

        selected_element
    }
}

impl Adjust {
    ///
    /// Creates the Adjust tool
    ///
    pub fn new() -> Adjust {
        Adjust { 
        }
    }

    ///
    /// Reads the control points for the selected region
    ///
    fn control_points_for_selection<Anim: 'static+EditableAnimation>(flo_model: &FloModel<Anim>) -> Vec<AdjustControlPoint> {
        // Get references to the bits of the model we need
        let selected        = flo_model.selection().selected_elements.get();
        let current_frame   = flo_model.frame().frame.get();

        // Need the selected elements and the current frame
        if let Some(current_frame) = current_frame.as_ref() {
            selected.iter()
                .flat_map(|element_id|                      current_frame.element_with_id(*element_id).map(|elem| (*element_id, elem)))
                .map(|(element_id, element)|                (element_id, current_frame.apply_properties_for_element(&element, Arc::new(VectorProperties::default())), element))
                .map(|(element_id, properties, element)|    (element_id, element.control_points(&*properties)))
                .flat_map(|(element_id, control_points)| {
                    control_points.into_iter()
                        .enumerate()
                        .map(move |(index, control_point)| AdjustControlPoint { 
                            owner:          element_id, 
                            index:          index,
                            control_point:  control_point
                        })
                })
                .collect()
        } else {
            vec![]
        }
    }

    ///
    /// Writes out a control point sprite for a bezier point
    ///
    fn declare_bezier_point_sprite(sprite_id: SpriteId, selected: bool) -> Vec<Draw> {
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
    fn declare_control_point_sprite(sprite_id: SpriteId) -> Vec<Draw> {
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
    fn drawing_for_selection_preview<Anim: 'static+EditableAnimation>(flo_model: &FloModel<Anim>) -> Vec<Draw> {
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
    fn drawing_for_control_points(control_points: &Vec<AdjustControlPoint>, selected_control_points: &HashSet<AdjustControlPointId>) -> Vec<Draw> {
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
    async fn render_selection_path<Anim: 'static+EditableAnimation>(actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>, control_points: BindRef<Arc<Vec<AdjustControlPoint>>>, selected_control_points: BindRef<HashSet<AdjustControlPointId>>) {
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
    async fn drag_control_points<Anim: 'static+EditableAnimation>(state: &mut AdjustToolState<Anim>, selected_control_points: &HashSet<AdjustControlPointId>, initial_event: Painting) {
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

    ///
    /// Starts a drag if the user moves far enough away from their current position (returning true if a drag was started)
    ///
    async fn maybe_drag<'a, Anim, ContinueFn, DragFuture>(state: &'a mut AdjustToolState<Anim>, initial_event: Painting, on_drag: ContinueFn) -> bool 
    where   Anim:       'static+EditableAnimation,
            DragFuture: 'a+Future<Output=()>,
            ContinueFn: FnOnce(&'a mut AdjustToolState<Anim>, Painting) -> DragFuture {
        // Distance the pointer should move to turn the gesture into a drag
        const DRAG_DISTANCE: f32    = 2.0;
        let (x1, y1)                = initial_event.location;

        while let Some(event) = state.input.next().await {
            match event {
                ToolInput::Paint(paint_event) => {
                    match paint_event.action {
                        PaintAction::Continue   |
                        PaintAction::Prediction  => {
                            if paint_event.pointer_id != initial_event.pointer_id {
                                // Changing pointer device cancels the drag
                                return false;
                            }

                            // If the pointer has moved more than DRAG_DISTANCE then switch to the 
                            let (x2, y2) = paint_event.location;
                            let (dx, dy) = (x1-x2, y1-y2);
                            let distance = ((dx*dx) + (dy*dy)).sqrt();

                            if distance >= DRAG_DISTANCE {
                                // Once the user moves more than a certain distance away, switch to dragging
                                on_drag(state, initial_event).await;
                                return true;
                            }
                        }

                        // Finishing the existing paint action cancels the drag
                        PaintAction::Finish => { return false; }

                        // If the gesture is cancelled, no drag takes place
                        PaintAction::Cancel => { return false; }

                        // If a new paint event starts, then it's likely that an event has been missed somewhere
                        PaintAction::Start  => { return false; }
                    }
                }

                _ => { }
            }
        }

        false
    }

    ///
    /// The user has begun a paint action on the canvas
    ///
    async fn click_or_drag_something<Anim: 'static+EditableAnimation>(state: &mut AdjustToolState<Anim>, initial_event: Painting) {
        // Do nothing if this isn't a paint start event
        if initial_event.action != PaintAction::Start {
            return;
        }

        // Find the control point that was clicked on, and update the selected control point set if one is found
        if let Some(dragged_control_points) = state.drag_control_points(initial_event.location.0 as f64, initial_event.location.1 as f64) {
            // Drag this handle instead of the selected control point
            Self::drag_control_points(state, &dragged_control_points, initial_event).await;
        } else if let Some(clicked_control_point) = state.control_point_at_position(initial_event.location.0 as f64, initial_event.location.1 as f64) {
            // The user has clicked on a control point
            let selected_control_points = state.selected_control_points.get();
            let mut drag_immediate      = true;

            if !selected_control_points.contains(&clicked_control_point) {
                if initial_event.modifier_keys == vec![ModifierKey::Shift] {
                    // Add to the selected control points
                    state.selected_control_points.set(iter::once(clicked_control_point).chain(selected_control_points.iter().cloned()).collect());
                    drag_immediate = false;
                } else {
                    // Select this control point
                    state.selected_control_points.set(iter::once(clicked_control_point).collect());
                    drag_immediate = false;
                }
            } else if initial_event.modifier_keys == vec![ModifierKey::Shift] {
                // Remove from the selected control points (reverse of the 'add' operation above)
                state.selected_control_points.set(selected_control_points.iter().filter(|cp| cp != &&clicked_control_point).cloned().collect());
                drag_immediate = false;
            }

            // Try to drag the control point: immediately if the user re-clicked an already selected control point, or after a delay if not
            let selected_control_points = state.selected_control_points.get();
            if drag_immediate {
                // Selection hasn't changed: treat as an immediate drag operation
                Self::drag_control_points(state, &selected_control_points, initial_event).await;
            } else {
                // Selection has changed: drag is 'sticky'
                Self::maybe_drag(state, initial_event, move |state, initial_event| async move { Self::drag_control_points(state, &selected_control_points, initial_event).await; }).await;
            }
        
        } else if let Some(selected_element) = state.element_at_position(initial_event.location.0 as f64, initial_event.location.1 as f64) {
            // The user hasn't clicked on a control point but has clicked on another element that we could edit

            if initial_event.modifier_keys != vec![ModifierKey::Shift] {
                // Holding down shift will toggle the element's selection state
                // TODO: if the user drags an edge of an existing element, transform using edge dragging instead of changing the selection
                state.flo_model.selection().clear_selection();
                state.flo_model.selection().selected_path.set(None);
                state.selected_control_points.set(HashSet::new());
            }

            state.flo_model.selection().toggle(selected_element);
        }
    }

    ///
    /// The main input loop for the adjust tool
    ///
    fn handle_input<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>, control_points: BindRef<Arc<Vec<AdjustControlPoint>>>, selected_control_points: Binding<HashSet<AdjustControlPointId>>) -> impl Future<Output=()>+Send {
        async move {
            let mut state = AdjustToolState {
                input:                      input,
                actions:                    actions,
                flo_model:                  flo_model, 
                control_points:             control_points,
                selected_control_points:    selected_control_points
            };

            while let Some(input_event) = state.input.next().await {
                match input_event {
                    ToolInput::Paint(paint_event) => {
                        Self::click_or_drag_something(&mut state, paint_event).await;
                    },

                    // Ignore other events
                    _ => { }
                }
            }
        }
    }

    ///
    /// Runs the adjust tool
    ///
    fn run<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>) -> impl Future<Output=()>+Send {
        async move {
            // Create a control points binding
            let model                   = flo_model.clone();
            let control_points          = computed(move || Arc::new(Self::control_points_for_selection(&*model)));
            let control_points          = BindRef::from(control_points);
            let selected_control_points = Binding::new(HashSet::new());

            // Declare the sprites for the adjust tool
            actions.send_actions(vec![ToolAction::Overlay(OverlayAction::Draw(Self::declare_bezier_point_sprite(SPRITE_BEZIER_POINT, false)))],);
            actions.send_actions(vec![ToolAction::Overlay(OverlayAction::Draw(Self::declare_bezier_point_sprite(SPRITE_SELECTED_BEZIER_POINT, true)))]);
            actions.send_actions(vec![ToolAction::Overlay(OverlayAction::Draw(Self::declare_control_point_sprite(SPRITE_BEZIER_CONTROL_POINT)))]);

            // Task that renders the selection path whenever it changes
            let render_selection_path   = Self::render_selection_path(actions.clone(), flo_model.clone(), control_points.clone(), BindRef::from(selected_control_points.clone()));

            // Task to handle the input from the user
            let handle_input            = Self::handle_input(input, actions, Arc::clone(&flo_model), control_points, selected_control_points);

            // Finish when either of the futures finish
            future::select_all(vec![render_selection_path.boxed(), handle_input.boxed()]).await;
        }
    }
}

impl<Anim: 'static+EditableAnimation> Tool<Anim> for Adjust {
    type ToolData   = ();
    type Model      = AdjustModel;

    fn tool_name(&self) -> String { "Adjust".to_string() }

    fn image(&self) -> Option<Image> { Some(svg_static(include_bytes!("../../svg/tools/adjust.svg"))) }

    fn create_model(&self, flo_model: Arc<FloModel<Anim>>) -> AdjustModel { 
        AdjustModel {
            future:         Mutex::new(ToolFuture::new(move |input, actions| { Self::run(input, actions, Arc::clone(&flo_model)) }))
        }
    }

    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &AdjustModel) -> Option<Arc<dyn Controller>> {
        Some(Arc::new(AdjustMenuController::new()))
    }

    ///
    /// Returns a stream containing the actions for the view and tool model for the select tool
    ///
    fn actions_for_model(&self, _flo_model: Arc<FloModel<Anim>>, tool_model: &AdjustModel) -> BoxStream<'static, ToolAction<()>> {
        tool_model.future.lock().unwrap().actions_for_model()
    }

    fn actions_for_input<'a>(&'a self, _flo_model: Arc<FloModel<Anim>>, tool_model: &AdjustModel, _data: Option<Arc<()>>, input: Box<dyn 'a+Iterator<Item=ToolInput<()>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<()>>> {
        Box::new(tool_model.future.lock().unwrap().actions_for_input(input).into_iter())
    }
}
