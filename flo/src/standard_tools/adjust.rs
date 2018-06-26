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

use std::sync::*;
use std::collections::HashSet;

///
/// Data for the Adjust tool
/// 
#[derive(Clone)]
pub struct AdjustData {
    /// The current frame
    frame: Option<Arc<dyn Frame>>,

    // The current set of selected elements
    selected_elements: Arc<HashSet<ElementId>>,
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
    fn selected_element_properties<SelectedElements, FrameBinding>(selected_elements: SelectedElements, frame: FrameBinding) -> impl Bound<Arc<Vec<(Vector, Arc<VectorProperties>)>>>
    where SelectedElements: 'static+Bound<Arc<HashSet<ElementId>>>, FrameBinding: 'static+Bound<Option<Arc<dyn Frame>>> {
        computed(move || {
            // Unwrap the bindings
            let frame                   = frame.get();
            let selected_elements       = selected_elements.get();

            // Go through all of the elements and store the properties for any that are marked as selected 
            let mut result: Vec<(Vector, Arc<VectorProperties>)>              = vec![];
            let mut current_properties  = Arc::new(VectorProperties::default());

            if let Some(frame) = frame {
                if let Some(elements) = frame.vector_elements() {
                    for element in elements {
                        current_properties = element.update_properties(current_properties);

                        if selected_elements.contains(&element.id()) {
                            result.push((element, Arc::clone(&current_properties)));
                        }
                    }
                }
            }   

            Arc::new(result)
        })
    }

    ///
    /// Creates an action stream that draws control points for the selection in the specified models
    /// 
    fn draw_control_point_overlay<Anim: 'static+Animation, FrameBinding: 'static+Bound<Option<Arc<dyn Frame>>>>(flo_model: Arc<FloModel<Anim>>, frame: FrameBinding) -> impl Stream<Item=ToolAction<AdjustData>, Error=()> {
        // Collect the selected elements into a hash set
        let selected_elements   = flo_model.selection().selected_element.clone();
        let selected_elements   = computed(move || Arc::new(selected_elements.get().into_iter().collect::<HashSet<_>>()));

        // Get the properties for the selected elements
        let selected_elements   = Self::selected_element_properties(selected_elements, frame);

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

        // Also track the selected elements
        let selected_elements   = flo_model.selection().selected_element.clone();

        // Draw control points when the frame changes
        let draw_control_points = Self::draw_control_point_overlay(flo_model, current_frame.clone());

        // Build the model from the current frame and selected elements
        let update_adjust_data = follow(computed(move || (current_frame.get(), selected_elements.get())))
            .map(|(frame, selected_elements)| {
                ToolAction::Data(AdjustData {
                    frame:              frame,
                    selected_elements:  Arc::new(selected_elements.into_iter().collect())
                })
            });
        
        // Actions are to update the data or draw the control points
        Box::new(update_adjust_data.select(draw_control_points))
    }

    fn actions_for_input<'a>(&'a self, _flo_model: Arc<FloModel<Anim>>, data: Option<Arc<AdjustData>>, input: Box<dyn 'a+Iterator<Item=ToolInput<AdjustData>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<AdjustData>>> {
        let mut data = data;

        // Process the input
        for input in input {
            match input {
                ToolInput::Data(new_data) => {
                    // Keep tracking the data as it changes
                    data = Some(new_data)
                },

                _ => ()
            }
        }

        // No actions
        Box::new(vec![].into_iter())
    }
}
