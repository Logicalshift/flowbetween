use super::super::menu::*;
use super::super::tools::*;
use super::super::model::*;
use super::super::style::*;

use ui::*;
use canvas::*;
use binding::*;
use animation::*;

use futures::*;
use futures::stream;
use itertools::*;

use std::f32;
use std::sync::*;
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
    /// Draws an element control point
    /// 
    pub fn draw_control_point(cp: &ControlPoint) -> Vec<Draw> {
        let mut draw = vec![];

        draw.new_path();

        match cp {
            ControlPoint::BezierPoint(x, y) => {
                draw.circle(*x, *y, 5.0);
                draw.fill_color(CP_BEZIER);
                draw.fill();
                draw.stroke();
            },

            ControlPoint::BezierControlPoint(x, y) => {
                draw.rect(x-3.0, y-3.0, x+3.0, y+3.0);
                draw.fill_color(CP_BEZIER_CP);
                draw.fill();
                draw.stroke();
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
        let paths = element.to_path(properties);

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

        let control_points = element.control_points();

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

        // Draw the control points themselves
        draw.stroke_color(SELECTION_OUTLINE);
        draw.line_width_pixels(1.0);

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
        let selected_elements   = flo_model.selection().selected_element.clone();
        let frame               = flo_model.frame().frame.clone();

        // Create a computed binding
        BindRef::new(&computed(move || {
            // Need the selected elements and the current frame
            let selected        = selected_elements.get();
            let current_frame   = frame.get();

            let control_points  = selected.into_iter()
                .map(move |element_id|              (element_id, current_frame.as_ref().and_then(|frame| frame.element_with_id(element_id))))
                .map(|(element_id, maybe_element)|  (element_id, maybe_element.map(|element| element.control_points()).unwrap_or_else(|| vec![])))
                .flat_map(|(element_id, control_points)| {
                    control_points.into_iter()
                        .enumerate()
                        .map(move |(index, control_point)| (element_id, index, control_point.position()))
                })
                .collect();

            // Final result
            Arc::new(control_points)
        }))
    }

    ///
    /// Creates an action stream that draws control points for the selection in the specified models
    /// 
    fn draw_control_point_overlay<Anim: 'static+Animation>(flo_model: Arc<FloModel<Anim>>) -> impl Stream<Item=ToolAction<AdjustData>, Error=()> {
        // Collect the selected elements into a hash set
        let selected_elements   = flo_model.selection().selected_element.clone();
        let selected_elements   = computed(move || Arc::new(selected_elements.get().into_iter().collect::<HashSet<_>>()));

        // Get the properties for the selected elements
        let selected_elements   = Self::selected_element_properties(selected_elements, &*flo_model);

        // Redraw the selected elements overlay layer every time the frame or the selection changes
        follow(selected_elements)
            .map(|selected_elements| {
                let mut draw_control_points = vec![];
                
                // Clear the layer we're going to draw the control points on
                draw_control_points.layer(0);
                draw_control_points.clear_layer();

                // Draw the control points for the selected elements
                for (vector, properties) in selected_elements.iter() {
                    draw_control_points.extend(Self::control_points_for_element(&**vector, &*properties));
                }

                // Generate the actions
                vec![ToolAction::Overlay(OverlayAction::Draw(draw_control_points))]
            })
            .map(|actions| stream::iter_ok(actions.into_iter()))
            .flatten()
    }
    
    ///
    /// Generates the tool actions for a painting action
    /// 
    fn paint(&self, painting: Painting, data: &AdjustData) -> Vec<ToolAction<AdjustData>> {
        let state           = data.state.get();
        let paint_action    = painting.action;

        match (state, paint_action) {
            // A start paint action might change the selection or start dragging a control point
            (_, PaintAction::Start) => {
                if let Some((cp_index, distance)) = data.nearest_control_point_index(painting.location) {
                    if distance < 8.0 {
                        // Start dragging this control point
                        let &(element_id, index, _pos) = &data.control_points[cp_index];
                        
                        data.state.clone().set(AdjustAction::DragControlPoint(element_id, index, painting.location, painting.location));
                    }
                }

                // No tool actions to perform
                vec![]
            },

            (AdjustAction::DragControlPoint(element_id, index, from, to), PaintAction::Continue) => {
                // Continue the control point drag by updating the 'to' location
                data.state.clone().set(AdjustAction::DragControlPoint(element_id, index, from, painting.location));

                // No tool actions to perform
                vec![]
            },

            // Default 'paint end' action is to reset to the 'no action' state
            (_, PaintAction::Finish) |
            (_, PaintAction::Cancel) => {
                // Reset the action back to 'no action'
                data.state.clone().set(AdjustAction::NoAction);
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
    fn actions_for_model(&self, flo_model: Arc<FloModel<Anim>>, _tool_model: &()) -> Box<dyn Stream<Item=ToolAction<AdjustData>, Error=()>+Send> {
        // Create a binding that works out the frame for the currently selected layer
        let current_frame   = flo_model.frame().frame.clone();

        // State is initially 'no action'
        let adjust_state = bind(AdjustAction::NoAction);

        // Also track the selected elements
        let selected_elements   = flo_model.selection().selected_element.clone();
        let control_points      = Self::control_points(&*flo_model);

        // Draw control points when the frame changes
        let draw_control_points = Self::draw_control_point_overlay(flo_model);

        // Build the model from the current frame and selected elements
        let update_adjust_data = follow(computed(move || (current_frame.get(), selected_elements.get(), control_points.get())))
            .map(move |(frame, selected_elements, control_points)| {
                ToolAction::Data(AdjustData {
                    frame:              frame,
                    state:              adjust_state.clone(),
                    selected_elements:  Arc::new(selected_elements.into_iter().collect()),
                    control_points:     control_points
                })
            });
        
        // Actions are to update the data or draw the control points
        Box::new(update_adjust_data.select(draw_control_points))
    }

    fn actions_for_input<'a>(&'a self, _flo_model: Arc<FloModel<Anim>>, data: Option<Arc<AdjustData>>, input: Box<dyn 'a+Iterator<Item=ToolInput<AdjustData>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<AdjustData>>> {
        let mut data = data;
        let mut actions = vec![];

        // Process the input
        for input in input {
            match input {
                ToolInput::Data(new_data) => {
                    // Keep tracking the data as it changes
                    data = Some(new_data);
                },

                ToolInput::Paint(painting) => {
                    if let Some(data) = data.as_ref() {
                        actions.extend(self.paint(painting, &**data));
                    }
                }

                _ => ()
            }
        }

        // No actions
        Box::new(actions.into_iter())
    }
}
