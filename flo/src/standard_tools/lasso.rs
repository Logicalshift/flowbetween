use crate::tools::*;
use crate::model::*;
use crate::style::*;

use flo_ui::*;
use flo_animation::*;
use flo_canvas::*;
use flo_curves::*;
use flo_curves::bezier::*;

use futures::prelude::*;
use futures::stream::{BoxStream};

use std::sync::*;

const LAYER_PREVIEW: u32 = 0;

///
/// The data stored for the Lasso tool
///
pub struct LassoModel {
    future: Mutex<ToolFuture>
}

///
/// The lasso tool
///
pub struct Lasso { }

impl Lasso {
    ///
    /// Creates a new lasso tool
    ///
    pub fn new() -> Lasso {
        Lasso { }
    }

    ///
    /// Fits a bezier path to a series of points
    ///
    fn fit_path<PointIter: Iterator<Item=(f32, f32)>>(points: PointIter) -> Option<Vec<Curve<PathPoint>>> {
        // Convert points to Coord2s
        let mut points          = points.map(|point| PathPoint::from(point));

        // Average points that are very close together so we don't overdo the curve fitting
        const MIN_DISTANCE: f64 = 2.0;

        let mut averaged_points = vec![];
        let last_point          = points.next();
        let mut last_point      = if let Some(last_point) = last_point { last_point } else { return None; };

        averaged_points.push(last_point);

        while let Some(point) = points.next() {
            // If the distance between this point and the last one is below a threshold, average them together
            let distance = last_point.distance_to(&point);

            if distance < MIN_DISTANCE {
                // Average this point with the previous average
                let num_averaged    = averaged_points.len();
                let current_average = averaged_points[num_averaged-1];
                let averaged_point  = (current_average + last_point) * 0.5;

                // Update the earlier point (and don't update last_point: we'll keep averaging until we find a new point far enough away)
                averaged_points[num_averaged-1] = averaged_point;
            } else {
                // Keep this point
                averaged_points.push(point);

                // Update the last point
                last_point = point;
            }
        }

        // Fit the points to make a path
        let path = fit_curve::<Curve<PathPoint>>(&averaged_points, 1.0);
        return path;
    }

    ///
    /// Returns how a preview path should be rendered
    ///
    fn drawing_for_path(path: Option<Vec<Curve<PathPoint>>>) -> Vec<Draw> {
        // Result is a drawing
        let mut path_drawing = vec![];

        // Start by clearing the preview layer
        path_drawing.layer(LAYER_PREVIEW);
        path_drawing.clear_layer();

        if let Some(path) = path {
            // Set up to render the path
            path_drawing.new_path();

            // Render the path points
            let initial_point = if path.len() == 0 {
                PathPoint::new(0.0, 0.0)
            } else {
                path[0].start_point()
            };

            path_drawing.move_to(initial_point.x(), initial_point.y());
            for curve in path.iter() {
                let (cp1, cp2)  = curve.control_points();
                let end_point   = curve.end_point();

                path_drawing.bezier_curve_to(end_point.x(), end_point.y(), cp1.x(), cp1.y(), cp2.x(), cp2.y());
            }

            path_drawing.close_path();

            // Render twice to generate the selection effect
            path_drawing.line_width_pixels(3.0);
            path_drawing.stroke_color(SELECTION_OUTLINE);
            path_drawing.stroke();

            path_drawing.line_width_pixels(1.0);
            path_drawing.stroke_color(SELECTION_BBOX);
            path_drawing.stroke();
        }

        path_drawing
    }

    ///
    /// After the user starts drawing, selects an area on the canvas
    ///
    pub async fn select_area(initial_event: Painting, input: &mut ToolInputStream<()>, actions: &mut ToolActionPublisher<()>)  {
        use self::ToolInput::*;

        // Start with a point that's just at the initial location
        let mut points              = vec![initial_event.location];
        let mut predicted_points    = vec![];

        // Read input until the user releases the mouse pointer
        while let Some(input) = input.next().await {
            match input {
                Paint(next_point) => {
                    // Only track events corresponding to the same pointer device as the initial action
                    if next_point.pointer_id != initial_event.pointer_id {
                        continue;
                    }

                    if next_point.action == PaintAction::Prediction {
                        // Add to the predicted points
                        predicted_points.push(next_point.location);
                    } else {
                        // Add to the list of points
                        predicted_points.drain(..);
                        points.push(next_point.location);
                    }

                    // Stop if the actgion is cancelled
                    if next_point.action == PaintAction::Cancel {
                        actions.send_actions(vec![
                            ToolAction::Overlay(OverlayAction::Draw(vec![Draw::Layer(LAYER_PREVIEW), Draw::ClearLayer]))
                        ]);
                        return;
                    }

                    // Fit a path to the points being selected
                    let select_path     = Self::fit_path(points.iter().cloned().chain(predicted_points.iter().cloned()));

                    // Draw the selection path
                    let select_drawing  = Self::drawing_for_path(select_path);
                    actions.send_actions(vec![
                        ToolAction::Overlay(OverlayAction::Draw(select_drawing))
                    ]);

                    // Return the resulting path if the action completes
                    if next_point.action == PaintAction::Finish {
                        actions.send_actions(vec![
                            ToolAction::Overlay(OverlayAction::Draw(vec![Draw::Layer(LAYER_PREVIEW), Draw::ClearLayer]))
                        ]);
                        return;
                    }
                }

                Deselect    => { break; }
                _           => { }
            }
        }
    }

    ///
    /// Implementation of the logic for the lasso tool
    ///
    pub async fn run(input: ToolInputStream<()>, actions: ToolActionPublisher<()>) {
        use self::ToolInput::*;

        let mut input   = input;
        let mut actions = actions;

        while let Some(input_event) = input.next().await {
            // Main input loop
            match input_event {
                Deselect        => { break; }
                Select          => { }
                Data(_)         => { }
                PaintDevice(_)  => { }

                Paint(painting) => { 
                    Self::select_area(painting, &mut input, &mut actions).await;
                }
            }
        }
    }
}

impl<Anim: 'static+EditableAnimation+Animation> Tool<Anim> for Lasso {
    ///
    /// Represents data for the tool at a point in time (typically a snapshot
    /// of the model)
    ///
    type ToolData = ();

    ///
    /// The type of the model used by the UI elements of this tool
    ///
    type Model = LassoModel;

    ///
    /// Retrieves the name of this tool
    ///
    fn tool_name(&self) -> String { 
        "Lasso".to_string()
    }

    ///
    /// Retrieves the image that represents this tool in the toolbar
    ///
    fn image(&self) -> Option<Image> {
        Some(svg_static(include_bytes!("../../svg/tools/lasso.svg")))
    }

    ///
    /// Creates a new instance of the UI model for this tool
    ///
    fn create_model(&self, flo_model: Arc<FloModel<Anim>>) -> Self::Model {
        LassoModel {
            future: Mutex::new(ToolFuture::new(move |input, actions| { Self::run(input, actions) }))
        }
    }

    ///
    /// Creates the menu controller for this tool (or None if this tool has no menu controller)
    ///
    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &Self::Model) -> Option<Arc<dyn Controller>> {
        None
    }

    ///
    /// Returns a stream of tool actions that result from changes to the model
    ///
    fn actions_for_model(&self, _flo_model: Arc<FloModel<Anim>>, tool_model: &Self::Model) -> BoxStream<'static, ToolAction<Self::ToolData>> {
        tool_model.future.lock().unwrap().actions_for_model()
    }

    ///
    /// Converts a set of tool inputs into the corresponding actions that should be performed
    ///
    fn actions_for_input<'a>(&'a self, flo_model: Arc<FloModel<Anim>>, tool_model: &Self::Model, data: Option<Arc<Self::ToolData>>, input: Box<dyn 'a+Iterator<Item=ToolInput<Self::ToolData>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<Self::ToolData>>> {
        Box::new(tool_model.future.lock().unwrap().actions_for_input(input).into_iter())
    }
}

