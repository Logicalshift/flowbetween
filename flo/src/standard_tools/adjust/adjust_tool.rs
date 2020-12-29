use super::constants::*;

use super::adjust_edge::*;
use super::adjust_state::*;
use super::adjust_model::*;
use super::adjust_control_point::*;

use super::super::select::*;

use crate::menu::*;
use crate::tools::*;
use crate::model::*;

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
use std::collections::{HashSet};

///
/// The Adjust tool, which alters control points and lines
///
pub struct Adjust { }

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
    /// Performs a drag on an edge
    ///
    async fn drag_edge<Anim: 'static+EditableAnimation>(state: &mut AdjustToolState<Anim>, selected_edge: AdjustEdgePoint, initial_event: Painting) {
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

    ///
    /// Starts a drag if the user moves far enough away from their current position (returning true if a drag was started)
    ///
    async fn maybe_drag<'a, Anim, ContinueFn, DragFuture>(state: &'a mut AdjustToolState<Anim>, initial_event: Painting, on_drag: ContinueFn) -> bool 
    where   Anim:       'static+EditableAnimation,
            DragFuture: 'a+Future<Output=()>,
            ContinueFn: FnOnce(&'a mut AdjustToolState<Anim>, Painting) -> DragFuture {
        // Distance the pointer should move to turn the gesture into a drag
        const DRAG_DISTANCE: f32    = (MIN_DISTANCE as f32) / 2.0;
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

        // A few behaviours are possible:
        //  * Dragging a handle of the selected control point (if there's only one)
        //  * Dragging a selected control point to move it
        //  * Clicking an edge to select the control points on either side or to drag it to a new position
        //  * Clicking on an unselected control point to select it (and optionally move it if dragged far enough)
        //  * Clicking on an unselected element to select it and thus show its control points
        //
        // Shift can be used to select extra elements or control points

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

        } else if let Some(selected_edge) = state.curve_at_position(initial_event.location.0 as f64, initial_event.location.1 as f64) {

            // The user has clicked on an edge
            if initial_event.modifier_keys != vec![ModifierKey::Shift] {
                // Holding down shift will toggle the element's selection state
                state.selected_control_points.set(HashSet::new());
            }

            // Select the start and end point of the edge
            let mut selected_control_points = state.selected_control_points.get();

            selected_control_points.insert(selected_edge.start_point.clone());
            selected_control_points.insert(selected_edge.end_point.clone());

            state.selected_control_points.set(selected_control_points);

            // Drag the edge if the user moves the cursor far enough
            Self::maybe_drag(state, initial_event, move |state, initial_event| async move { Self::drag_edge(state, selected_edge, initial_event).await; }).await;

        } else if let Some(selected_element) = state.element_at_position(initial_event.location.0 as f64, initial_event.location.1 as f64) {
            
            // The user hasn't clicked on a control point but has clicked on another element that we could edit

            if initial_event.modifier_keys != vec![ModifierKey::Shift] {
                // Holding down shift will toggle the element's selection state
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

    fn image(&self) -> Option<Image> { Some(svg_static(include_bytes!("../../../svg/tools/adjust.svg"))) }

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
