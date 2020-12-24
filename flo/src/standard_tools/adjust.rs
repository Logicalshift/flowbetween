use super::select::*;

use crate::menu::*;
use crate::tools::*;
use crate::model::*;
use crate::style::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;
use futures::stream::{BoxStream};

use std::f32;
use std::f64;
use std::iter;
use std::sync::*;
use std::collections::{HashSet};

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

        for cp in self.control_points.get().iter() {
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
    /// The user has begun a paint action on the canvas
    ///
    async fn click_or_drag_something<Anim: 'static+EditableAnimation>(state: &mut AdjustToolState<Anim>, initial_event: Painting) {
        // Do nothing if this isn't a paint start event
        if initial_event.action != PaintAction::Start {
            return;
        }

        // Find the control point that was clicked on, and update the selected control point set if one is found
        // TODO: handle the case where there's a single selected point and allow the bezier points to be dragged
        let clicked_control_point = state.control_point_at_position(initial_event.location.0 as f64, initial_event.location.1 as f64);

        if let Some(clicked_control_point) = clicked_control_point {
            // The user has clicked on a control point
            let selected_control_points = state.selected_control_points.get();

            if !selected_control_points.contains(&clicked_control_point) {
                if initial_event.modifier_keys == vec![ModifierKey::Shift] {
                    // Add to the selected control points
                    state.selected_control_points.set(iter::once(clicked_control_point).chain(selected_control_points.iter().cloned()).collect());
                } else {
                    // Select this control point
                    state.selected_control_points.set(iter::once(clicked_control_point).collect());
                }
            } else if initial_event.modifier_keys == vec![ModifierKey::Shift] {
                // Remove from the selected control points (reverse of the 'add' operation above)
                state.selected_control_points.set(selected_control_points.iter().filter(|cp| cp != &&clicked_control_point).cloned().collect());
            }

            // TODO: Try to drag the control point: immediately if the user re-clicked an already selected control point, or after a delay if not
        
        } else if let Some(selected_element) = state.element_at_position(initial_event.location.0 as f64, initial_event.location.1 as f64) {
            // The user hasn't clicked on a control point but has clicked on another element that we could edit

            if initial_event.modifier_keys != vec![ModifierKey::Shift] {
                // Holding down shift will toggle the element's selection state
                // TODO: if the user drags an edge of an existing element, transform using edge dragging instead of changing the selection
                state.flo_model.selection().clear_selection();
                state.flo_model.selection().selected_path.set(None);
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
    /// Generates the selection preview drawing actions from a model
    ///
    fn drawing_for_selection_preview<Anim: 'static+EditableAnimation>(flo_model: &FloModel<Anim>) -> Vec<Draw> {
        // Determine the selected elements from the model
        let current_frame           = flo_model.frame().frame.get();
        let selected_elements       = flo_model.selection().selected_elements.get();

        // Fetch the elements from the frame and determine how to draw the highlight for them
        let mut selection_drawing   = vec![];
        let mut bounds              = Rect::empty();

        if let Some(current_frame) = current_frame.as_ref() {
            for selected_id in selected_elements.iter() {
                let element = current_frame.element_with_id(*selected_id);

                if let Some(element) = element {
                    // Update the properties according to this element
                    let properties  = current_frame.apply_properties_for_element(&element, Arc::new(VectorProperties::default()));

                    // Draw the highlight for it
                    let (drawing, bounding_box) = Select::highlight_for_selection(&element, &properties);
                    selection_drawing.extend(drawing);
                    bounds = bounds.union(bounding_box);
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

            for cp_index in 1..(control_points.len()-1) {
                let cp = &control_points[cp_index];
                let (x1, y1) = cp.control_point.position();

                if cp.owner == selected_control_point.owner && cp.index == selected_control_point.index {
                    // Draw the control points for this CP (preceding and following point)
                    let preceding = &control_points[cp_index-1];
                    let following = &control_points[cp_index+1];

                    drawing.line_width_pixels(1.0);
                    drawing.stroke_color(CP_LINES);

                    if preceding.control_point.is_control_point() {
                        let (x2, y2) = preceding.control_point.position();

                        drawing.new_path();
                        drawing.move_to(x1 as f32, y1 as f32);
                        drawing.line_to(x2 as f32, y2 as f32);
                        drawing.stroke();
                    }

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

            for cp_index in 1..(control_points.len()-1) {
                let cp = &control_points[cp_index];
                if cp.owner == selected_control_point.owner && cp.index == selected_control_point.index {
                    // Draw the control points for this CP (preceding and following point)
                    let preceding = &control_points[cp_index-1];
                    let following = &control_points[cp_index+1];

                    if preceding.control_point.is_control_point() {
                        let (x, y) = preceding.control_point.position();

                        drawing.sprite_transform(SpriteTransform::Identity);
                        drawing.sprite_transform(SpriteTransform::Translate(x as f32, y as f32));
                        drawing.draw_sprite(SPRITE_BEZIER_CONTROL_POINT);
                    }

                    if following.control_point.is_control_point() {
                        let (x, y) = following.control_point.position();

                        drawing.sprite_transform(SpriteTransform::Identity);
                        drawing.sprite_transform(SpriteTransform::Translate(x as f32, y as f32));
                        drawing.draw_sprite(SPRITE_BEZIER_CONTROL_POINT);
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
        let model               = flo_model.clone();
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
    fn actions_for_model(&self, flo_model: Arc<FloModel<Anim>>, tool_model: &AdjustModel) -> BoxStream<'static, ToolAction<()>> {
        tool_model.future.lock().unwrap().actions_for_model()
    }

    fn actions_for_input<'a>(&'a self, flo_model: Arc<FloModel<Anim>>, tool_model: &AdjustModel, _data: Option<Arc<()>>, input: Box<dyn 'a+Iterator<Item=ToolInput<()>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<()>>> {
        Box::new(tool_model.future.lock().unwrap().actions_for_input(input).into_iter())
    }
}
