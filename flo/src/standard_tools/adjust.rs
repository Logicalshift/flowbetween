use super::super::menu::*;
use super::super::tools::*;
use super::super::model::*;
use super::super::style::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;

use futures::*;
use futures::stream;
use futures::stream::{BoxStream};
use itertools::*;

use std::f32;
use std::sync::*;
use std::time::Duration;
use std::collections::HashSet;

///
/// The current action being performed by the adjust tool
///
#[derive(Clone, Copy, Debug, PartialEq)]
enum AdjustAction {
    /// The tool is idle
    NoAction,

    /// Selected an element
    Select,

    /// A control point is being adjusted
    DragControlPoint(ElementId, usize, (f32, f32), (f32, f32))
}

///
/// Data for the Adjust tool
///
#[derive(Clone)]
pub struct AdjustData {
    /// The current frame
    frame: Option<Arc<dyn Frame>>,

    /// The time of the current frame
    frame_time: Duration,

    /// The current state of this data
    state: Binding<AdjustAction>,

    // The current set of selected elements
    selected_elements: Arc<HashSet<ElementId>>,

    // The element, index and location of all of the control points
    control_points: Arc<Vec<(ElementId, usize, (f32, f32))>>
}

impl AdjustData {
    ///
    /// Finds the nearest control point to a particular location
    ///
    fn nearest_control_point_index(&self, location: (f32, f32)) -> Option<(usize, f32)> {
        let mut min_dist = f32::MAX;
        let mut cp_index = None;

        // Find the index of the closest control point
        for (index, cp) in self.control_points.iter().enumerate() {
            let pos             = cp.2;
            let diff_x          = location.0 - pos.0;
            let diff_y          = location.1 - pos.1;

            let dist_squared    = diff_x*diff_x + diff_y*diff_y;

            if dist_squared < min_dist {
                min_dist = dist_squared;
                cp_index = Some(index);
            }
        }

        // Return the index and the distance of the nearest control point
        cp_index.map(|index| (index, min_dist.sqrt()))
    }
}

///
/// The Adjust tool (adjusts control points of existing objects)
///
pub struct Adjust { }

impl Adjust {
    ///
    /// Creates a new instance of the Adjust tool
    ///
    pub fn new() -> Adjust {
        Adjust {}
    }

    ///
    /// Writes out a control point sprite for a bezier point
    ///
    fn declare_bezier_point_sprite(sprite_id: SpriteId) -> Vec<Draw> {
        let mut draw = vec![];

        draw.sprite(sprite_id);
        draw.clear_sprite();

        draw.stroke_color(SELECTION_OUTLINE);
        draw.line_width_pixels(1.0);

        draw.circle(0.0, 0.0, 5.0);
        draw.fill_color(CP_BEZIER);
        draw.fill();
        draw.stroke();

        draw
    }

    ///
    /// Writes out a control point sprite for a bezier control point
    ///
    fn declare_bezier_control_point_sprite(sprite_id: SpriteId) -> Vec<Draw> {
        let mut draw = vec![];

        draw.sprite(sprite_id);
        draw.clear_sprite();

        draw.stroke_color(SELECTION_OUTLINE);
        draw.line_width_pixels(1.0);

        draw.rect(0.0-3.0, 0.0-3.0, 0.0+3.0, 0.0+3.0);
        draw.fill_color(CP_BEZIER_CP);
        draw.fill();
        draw.stroke();

        draw
    }

    ///
    /// Draws an element control point
    ///
    pub fn draw_control_point(cp: &ControlPoint) -> Vec<Draw> {
        let mut draw = vec![];

        draw.new_path();

        match cp {
            ControlPoint::BezierPoint(x, y) => {
                draw.sprite_transform(SpriteTransform::Identity);
                draw.sprite_transform(SpriteTransform::Translate(*x, *y));
                draw.draw_sprite(SPRITE_BEZIER_POINT);
            },

            ControlPoint::BezierControlPoint(x, y) => {
                draw.sprite_transform(SpriteTransform::Identity);
                draw.sprite_transform(SpriteTransform::Translate(*x, *y));
                draw.draw_sprite(SPRITE_BEZIER_CONTROL_POINT);
            }
        };

        draw
    }

    ///
    /// Returns the drawing instructions for the control points for an element
    ///
    pub fn control_points_for_element(element: &dyn VectorElement, properties: &VectorProperties) -> Vec<Draw> {
        // Create the vector where the control points will be drawn
        let mut draw = vec![];

        // Outline the path
        let paths = element.to_path(properties, PathConversion::Fastest);

        if let Some(paths) = paths {
            draw.new_path();
            draw.extend(paths.into_iter().flat_map(|path| -> Vec<Draw> { (&path).into() }));

            draw.stroke_color(SELECTION_OUTLINE);
            draw.line_width_pixels(2.0);
            draw.stroke();

            draw.stroke_color(SELECTION_HIGHLIGHT);
            draw.line_width_pixels(0.5);
            draw.stroke();
        }

        let control_points = element.control_points(properties);

        /*
        // Draw the control point connecting lines
        draw.new_path();
        for (prev, next) in control_points.iter().tuple_windows() {
            match (prev, next) {
                (ControlPoint::BezierPoint(x1, y1), ControlPoint::BezierControlPoint(x2, y2)) |
                (ControlPoint::BezierControlPoint(x2, y2), ControlPoint::BezierPoint(x1, y1)) => {
                    draw.move_to(*x1, *y1);
                    draw.line_to(*x2, *y2);
                },

                _ => ()
            }
        }

        draw.line_width_pixels(2.0);
        draw.stroke_color(SELECTION_OUTLINE);
        draw.stroke();
        draw.line_width_pixels(1.0);
        draw.stroke_color(CP_LINES);
        draw.stroke();
        */

        // Draw the control points themselves
        for cp in control_points.iter() {
            draw.extend(Self::draw_control_point(cp));
        }

        draw
    }

    ///
    /// Creates a binding that contains the vector/vector properties for the currently selected elements in a frame
    ///
    fn selected_element_properties<SelectedElements, Anim>(selected_elements: SelectedElements, flo_model: &FloModel<Anim>) -> impl Bound<Arc<Vec<(Vector, Arc<VectorProperties>)>>>
    where SelectedElements: 'static+Bound<Arc<HashSet<ElementId>>>, Anim: 'static+Animation {
        // The elements binding contains the vector elements and their properties for the current frame
        let elements = flo_model.frame().elements.clone();

        computed(move || {
            // Unwrap the bindings
            let elements                = elements.get();
            let selected_elements       = selected_elements.get();

            // Filter to the selected elements only
            Arc::new(elements.iter()
                .filter(|(element, _)| selected_elements.contains(&element.id()))
                .cloned()
                .collect())
        })
    }

    ///
    /// Returns a binding that returns a list of all the control points in the current selection (ie, everything that can be dragged)
    ///
    fn control_points<Anim: 'static+Animation>(flo_model: &FloModel<Anim>) -> BindRef<Arc<Vec<(ElementId, usize, (f32, f32))>>> {
        // Get references to the bits of the model we need
        let selected_elements   = flo_model.selection().selected_elements.clone();
        let frame               = flo_model.frame().frame.clone();

        // Create a computed binding
        BindRef::new(&computed(move || {
            // Need the selected elements and the current frame
            let selected        = selected_elements.get();
            let current_frame   = frame.get();

            let control_points  = if let Some(current_frame) = current_frame.as_ref() {
                selected.iter()
                    .flat_map(|element_id|                      current_frame.element_with_id(*element_id).map(|elem| (*element_id, elem)))
                    .map(|(element_id, element)|                (element_id, current_frame.apply_properties_for_element(&element, Arc::new(VectorProperties::default())), element))
                    .map(|(element_id, properties, element)|    (element_id, element.control_points(&*properties)))
                    .flat_map(|(element_id, control_points)| {
                        control_points.into_iter()
                            .enumerate()
                            .map(move |(index, control_point)| (element_id, index, control_point.position()))
                    })
                    .collect()
            } else {
                vec![]
            };

            // Final result
            Arc::new(control_points)
        }))
    }

    ///
    /// Creates an action stream that draws control points for the selection in the specified models
    ///
    fn draw_control_point_overlay<Anim: 'static+Animation>(flo_model: Arc<FloModel<Anim>>, tool_state: BindRef<AdjustAction>) -> impl Stream<Item=ToolAction<AdjustData>> {
        // Collect the selected elements into a HashSet
        let selected_elements   = flo_model.selection().selected_elements.clone();
        let selected_elements   = computed(move || selected_elements.get());

        // Get the properties for the selected elements
        let selected_elements   = Self::selected_element_properties(selected_elements, &*flo_model);

        // We hide the element that is being dragged (it'll get drawn on the edit overlay layer)
        // TODO: this will update too frequently right now as we'll generate an event whenever the state changes
        let hidden_element      = computed(move || {
            match tool_state.get() {
                AdjustAction::DragControlPoint(element_id, _, _, _) => Some(element_id),
                _ => None
            }
        });

        // Redraw the selected elements overlay layer every time the frame or the selection changes
        follow(computed(move || (selected_elements.get(), hidden_element.get())))
            .map(|(selected_elements, hidden_element)| {
                let mut draw_control_points = vec![];

                // Clear the layer we're going to draw the control points on
                draw_control_points.layer(0);
                draw_control_points.clear_layer();
                draw_control_points.extend(Self::declare_bezier_point_sprite(SPRITE_BEZIER_POINT));
                draw_control_points.extend(Self::declare_bezier_control_point_sprite(SPRITE_BEZIER_CONTROL_POINT));
                draw_control_points.layer(0);

                // Draw the control points for the selected elements
                for (vector, properties) in selected_elements.iter() {
                    if Some(vector.id()) != hidden_element {
                        draw_control_points.extend(Self::control_points_for_element(&**vector, &*properties));
                    }
                }

                // Generate the actions
                vec![ToolAction::Overlay(OverlayAction::Draw(draw_control_points))]
            })
            .map(|actions| stream::iter(actions.into_iter()))
            .flatten()
    }

    ///
    /// Returns the control points as adjusted by an edit actions
    ///
    fn adjusted_control_points(to_edit: &Vector, properties: &VectorProperties, action: &AdjustAction) -> Vec<(f32, f32)> {
        // Fetch the control points for this element
        let control_points          = to_edit.control_points(properties);
        let mut new_control_points  = vec![];

        match action {
            // Dragging a single control point just updates that one control point
            AdjustAction::DragControlPoint(_, index, from, to) => {
                let (diff_x, diff_y)        = (to.0-from.0, to.1-from.1);

                // When moving a point on the line, we need to move its control points with it
                let should_move_neighbours  = !control_points[*index].is_control_point();

                for (this_index, cp) in control_points.into_iter().enumerate() {
                    let pos = cp.position();

                    if this_index == *index
                    || (((*index > 0 && this_index == *index-1) || this_index == *index+1) && should_move_neighbours) {
                        // Move this control point
                        new_control_points.push((pos.0+diff_x, pos.1+diff_y));
                    } else {
                        // Control point is unchanged
                        new_control_points.push(pos);
                    }
                }
            },

            // Default is no editing
            _ => {
                new_control_points = control_points.into_iter()
                    .map(|cp| cp.position())
                    .collect();
            }
        }

        new_control_points
    }

    ///
    /// Performs an editing action on a vector
    ///
    fn edit_vector(to_edit: &Vector, properties: &VectorProperties, action: &AdjustAction) -> Vector {
        // Fetch the new control points
        let new_control_points = Self::adjusted_control_points(to_edit, properties, action);

        // Return the vector updated with these control points
        to_edit.with_adjusted_control_points(new_control_points, properties)
    }

    ///
    /// Finds the adjust control points for a vector as they would be after the motions applied to an element are reversed
    ///
    /// These are the control points we need to store as the edited control points for an element.
    /// The element as we have it for editing (in `to_edit`) is how it is represented after the motions
    /// are applied.
    ///
    fn adjusted_control_points_before_motion<Anim: 'static+Animation>(flo_model: &FloModel<Anim>, data: &AdjustData, to_edit: &Vector, action: &AdjustAction) -> Vec<(f32, f32)> {
        // Get the properties for the element
        let mut properties = Arc::new(VectorProperties::default());
        if let Some(frame) = flo_model.frame().frame.get() {
            properties = frame.apply_properties_for_element(to_edit, properties);
        }

        // Apply the edit to the vector
        let mut adjusted = Self::edit_vector(to_edit, &*properties, action);

        // Return the control points for the adjusted vector
        adjusted.control_points(&*properties)
            .into_iter()
            .map(|cp| cp.position())
            .collect()
    }

    ///
    /// Returns the actions required to draw the editing overlay
    ///
    fn draw_edit_overlay<Anim: 'static+Animation>(flo_model: Arc<FloModel<Anim>>, tool_state: BindRef<AdjustAction>) -> impl Stream<Item=ToolAction<AdjustData>> {
        // Get the set of elements from the frame
        let elements    = flo_model.frame().elements.clone();

        // Create a single binding for the state of the editing overlay
        let edit_state  = computed(move || {
            let state = tool_state.get();

            match state {
                AdjustAction::DragControlPoint(_, _, _, _) =>  {
                    // Only fetch the elements while actually dragging an element
                    Some((state, elements.get()))
                },

                // No edit state
                _ => None
            }
        });

        // Draw an overlay layer as the edit state is updated
        follow(edit_state)
            .map(|edit_state| {
                match edit_state {
                    Some((AdjustAction::DragControlPoint(element_id, index, from, to), elements)) => {
                        // Draw this element in its new position
                        let mut draw_drag   = vec![];
                        let action          = AdjustAction::DragControlPoint(element_id, index, from, to);

                        // Clear the edit layer
                        draw_drag.layer(1);
                        draw_drag.clear_layer();
                        draw_drag.extend(Self::declare_bezier_point_sprite(SPRITE_BEZIER_POINT));
                        draw_drag.extend(Self::declare_bezier_control_point_sprite(SPRITE_BEZIER_CONTROL_POINT));
                        draw_drag.layer(1);

                        // Edit the elements
                        let elements_to_draw    = elements.iter().filter(|(vector, _)|          vector.id() == element_id);
                        let edited_elements     = elements_to_draw.map(|(vector, properties)|   (Self::edit_vector(vector, &*properties, &action), properties));
                        let edited_elements     = edited_elements.collect::<Vec<_>>();

                        // Draw them with their control points
                        let draw_control_points = edited_elements.iter()
                            .flat_map(|(vector, properties)| Self::control_points_for_element(&**vector, properties));

                        // Apply to the action
                        draw_drag.extend(draw_control_points);

                        // Draw the drag operation
                        vec![ToolAction::Overlay(OverlayAction::Draw(draw_drag))]
                    },

                    _ => {
                        // No edits to draw
                        let mut clear_layer = vec![];

                        // Clear the edit layer
                        clear_layer.layer(1);
                        clear_layer.clear_layer();

                        // Generate the actions
                        vec![ToolAction::Overlay(OverlayAction::Draw(clear_layer))]
                    }
                }
            })
            .map(|actions| stream::iter(actions.into_iter()))
            .flatten()
    }

    ///
    /// Generates the tool actions for a painting action
    ///
    fn paint<Anim: 'static+Animation>(&self, painting: Painting, data: &AdjustData, model: &FloModel<Anim>) -> Vec<ToolAction<AdjustData>> {
        let state           = data.state.get();
        let paint_action    = painting.action;

        match (state, paint_action) {
            // A start paint action might change the selection or start dragging a control point
            (_, PaintAction::Start) => {
                let mut started_drag    = false;
                let mut actions         = vec![];

                // If the user clicks on a control point, dragging that takes priority
                if let Some((cp_index, distance)) = data.nearest_control_point_index(painting.location) {
                    if distance < 8.0 {
                        // Start dragging this control point
                        let &(element_id, index, _pos) = &data.control_points[cp_index];

                        data.state.set(AdjustAction::DragControlPoint(element_id, index, painting.location, painting.location));
                        started_drag = true;
                    }
                }

                // If the user has not started a drag, consider changing the selection
                if !started_drag {
                    // Find the elements at this point
                    let frame       = model.frame();
                    let elements    = frame.elements_at_point(painting.location);

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

                    // Select the element if we found one
                    if let Some(selected_element) = selected_element {
                        actions.push(ToolAction::ClearSelection);
                        actions.push(ToolAction::Select(selected_element));
                    } else {
                        actions.push(ToolAction::ClearSelection);
                    }

                    // This is a selection action
                    data.state.set(AdjustAction::Select);
                }

                // Result is the list of actions
                actions
            },

            (AdjustAction::DragControlPoint(element_id, index, from, _to), PaintAction::Continue)   |
            (AdjustAction::DragControlPoint(element_id, index, from, _to), PaintAction::Prediction) => {
                // Continue the control point drag by updating the 'to' location
                data.state.set(AdjustAction::DragControlPoint(element_id, index, from, painting.location));

                // No tool actions to perform
                vec![]
            },

            (AdjustAction::DragControlPoint(element_id, index, from, _to), PaintAction::Finish) => {
                // Continue the control point drag by updating the final 'to' location
                let final_action = AdjustAction::DragControlPoint(element_id, index, from, painting.location);

                // Action should become 'no action'
                data.state.set(AdjustAction::NoAction);

                // Fetch the element that will be edited
                let vector              = data.frame.as_ref().and_then(|frame| frame.element_with_id(element_id));

                // Generate the edit action for this element
                let edit_element        = if let Some(vector) = vector {
                    let time_index          = data.frame.as_ref().map(|frame| frame.time_index()).unwrap_or(Duration::from_millis(0));
                    let new_control_points  = Self::adjusted_control_points_before_motion(model, data, &vector, &final_action);
                    vec![
                        ToolAction::Edit(AnimationEdit::Element(vec![element_id], ElementEdit::SetControlPoints(new_control_points, time_index))),
                        ToolAction::InvalidateFrame
                    ]
                } else {
                    // Element cannot be found in the frame so cannot be edited
                    vec![]
                };

                // Perform these actions
                edit_element
            },

            // Default 'paint end' action is to reset to the 'no action' state
            (_, PaintAction::Finish) |
            (_, PaintAction::Cancel) => {
                // Reset the action back to 'no action'
                data.state.set(AdjustAction::NoAction);
                vec![]
            },

            // Unknown state: we take no action as a result of this
            _ => vec![]
        }
    }
}

impl<Anim: 'static+Animation> Tool<Anim> for Adjust {
    type ToolData   = AdjustData;
    type Model      = ();

    fn tool_name(&self) -> String { "Adjust".to_string() }

    fn image_name(&self) -> String { "adjust".to_string() }

    fn create_model(&self, _flo_model: Arc<FloModel<Anim>>) -> () { }

    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &()) -> Option<Arc<dyn Controller>> {
        Some(Arc::new(AdjustMenuController::new()))
    }

    ///
    /// Returns a stream containing the actions for the view and tool model for the select tool
    ///
    fn actions_for_model(&self, flo_model: Arc<FloModel<Anim>>, _tool_model: &()) -> BoxStream<'static, ToolAction<AdjustData>> {
        // Create a binding that works out the frame for the currently selected layer
        let current_frame   = flo_model.frame().frame.clone();

        // State is initially 'no action'
        let adjust_state = bind(AdjustAction::NoAction);

        // Also track the selected elements
        let selected_elements   = flo_model.selection().selected_elements.clone();
        let control_points      = Self::control_points(&*flo_model);

        // Draw control points when the frame changes
        let draw_control_points = Self::draw_control_point_overlay(flo_model.clone(), BindRef::new(&adjust_state));

        // When the user is dragging an element, draw a preview of the final look of that element
        let draw_drag_result = Self::draw_edit_overlay(flo_model.clone(), BindRef::new(&adjust_state));

        // We need to know when the user is editing things (to compensate for motions etc)
        let frame_time = flo_model.timeline().current_time.clone();

        // Build the model from the current frame and selected elements
        let update_adjust_data = follow(computed(move || (current_frame.get(), selected_elements.get(), control_points.get(), frame_time.get())))
            .map(move |(frame, selected_elements, control_points, frame_time)| {
                ToolAction::Data(AdjustData {
                    frame:              frame,
                    frame_time:         frame_time,
                    state:              adjust_state.clone(),
                    selected_elements:  selected_elements.clone(),
                    control_points:     control_points
                })
            });

        // Actions are to update the data or draw the control points
        Box::pin(stream::select(stream::select(update_adjust_data, draw_control_points), draw_drag_result))
    }

    fn actions_for_input<'a>(&'a self, flo_model: Arc<FloModel<Anim>>, data: Option<Arc<AdjustData>>, input: Box<dyn 'a+Iterator<Item=ToolInput<AdjustData>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<AdjustData>>> {
        let mut data    = data;
        let mut actions = vec![];
        let input       = ToolInput::last_paint_actions_only(input);

        // Process the input
        for input in input {
            match input {
                ToolInput::Data(new_data) => {
                    // Keep tracking the data as it changes
                    data = Some(new_data);
                },

                ToolInput::Paint(painting) => {
                    if let Some(data) = data.as_ref() {
                        actions.extend(self.paint(painting, &**data, &*flo_model));
                    }
                }

                _ => ()
            }
        }

        // No actions
        Box::new(actions.into_iter())
    }
}
