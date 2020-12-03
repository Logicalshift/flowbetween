use super::select::*;

use crate::tools::*;
use crate::model::*;
use crate::style::*;

use flo_ui::*;
use flo_animation::*;
use flo_canvas::*;
use flo_curves::*;
use flo_curves::bezier::*;
use flo_curves::bezier::path::*;
use flo_binding::*;

use futures::prelude::*;
use futures::future;
use futures::stream::{BoxStream};

use std::sync::*;

/// Layer where the current selection is drawn
const LAYER_SELECTION: u32 = 0;

/// Layer where the preview of the region the user is dragging is drawn
const LAYER_PREVIEW: u32 = 1;

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
        let path = Curve::<PathPoint>::fit_from_points(&averaged_points, 2.0);
        return path;
    }

    ///
    /// Returns how a preview path should be rendered
    ///
    fn drawing_for_path(path: &Option<Vec<Curve<PathPoint>>>) -> Vec<Draw> {
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
    /// Creates an animation path from the iytoyt of fit_path
    ///
    fn path_for_path(path: &Vec<Curve<PathPoint>>) -> Path {
        // Start building a path
        let mut builder = BezierPathBuilder::<Path>::start(path[0].start_point());

        // Add the curves from the list
        for point in path.iter() {
            let ep          = point.end_point();
            let (cp1, cp2)  = point.control_points();

            builder = builder.curve_to((cp1, cp2), ep);
        }

        builder = builder.line_to(path[0].start_point());

        // Finish building the path
        let path = builder.build();
        let path = path_remove_interior_points(&vec![path], 0.01);
        let path = Path::from_paths(path.iter());

        path
    }

    ///
    /// A function that keeps the selected path binding rendered and up to date
    ///
    pub async fn render_selection_path(selected_path: BindRef<Option<Arc<Path>>>, actions: ToolActionPublisher<()>, layer: u32) {
        // Convert the binding to a stream
        let mut selected_path = follow(selected_path);

        // Re-render the selected path every time it changes
        while let Some(selected_path) = selected_path.next().await {
            // Draw the selected path
            let mut draw_selected_path = vec![];

            draw_selected_path.layer(layer);
            draw_selected_path.clear_layer();

            if let Some(selected_path) = selected_path {
                draw_selected_path.new_path();

                for path_component in selected_path.elements() {
                    use self::PathComponent::*;

                    match path_component {
                        Move(PathPoint { position: (x, y)} )        => draw_selected_path.move_to(x as f32, y as f32),
                        Line(PathPoint { position: (x, y)} )        => draw_selected_path.line_to(x as f32, y as f32),
                        Bezier(
                            PathPoint { position: (tx, ty) }, 
                            PathPoint { position: (cp1x, cp1y) }, 
                            PathPoint { position: (cp2x, cp2y) })   => draw_selected_path.bezier_curve_to(tx as f32, ty as f32, cp1x as f32, cp1y as f32, cp2x as f32, cp2y as f32),
                        Close                                       => draw_selected_path.close_path()
                    }
                }

                // Render twice to generate the selection effect
                draw_selected_path.line_width_pixels(3.0);
                draw_selected_path.stroke_color(SELECTION_OUTLINE);
                draw_selected_path.stroke();

                draw_selected_path.line_width_pixels(1.0);
                draw_selected_path.stroke_color(SELECTION_BBOX);
                draw_selected_path.stroke();
            }

            // Publish an action to draw the path
            actions.send_actions(vec![
                ToolAction::Overlay(OverlayAction::Draw(draw_selected_path))
            ]);
        }
    }

    ///
    /// After the user starts drawing, selects an area on the canvas
    ///
    pub async fn select_area(initial_event: Painting, input: &mut ToolInputStream<()>, actions: &mut ToolActionPublisher<()>) -> Option<Vec<Curve<PathPoint>>> {
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

                    // Stop if the action is cancelled
                    if next_point.action == PaintAction::Cancel {
                        actions.send_actions(vec![
                            ToolAction::Overlay(OverlayAction::Draw(vec![Draw::Layer(LAYER_PREVIEW), Draw::ClearLayer]))
                        ]);
                        return None;
                    }

                    // Fit a path to the points being selected
                    let select_path     = Self::fit_path(points.iter().cloned().chain(predicted_points.iter().cloned()));

                    // Draw the selection path
                    let select_drawing  = Self::drawing_for_path(&select_path);
                    actions.send_actions(vec![
                        ToolAction::Overlay(OverlayAction::Draw(select_drawing))
                    ]);

                    // Return the resulting path if the action completes
                    if next_point.action == PaintAction::Finish {
                        actions.send_actions(vec![
                            ToolAction::Overlay(OverlayAction::Draw(vec![Draw::Layer(LAYER_PREVIEW), Draw::ClearLayer]))
                        ]);
                        return select_path;
                    }
                }

                Deselect    => { break; }
                _           => { }
            }
        }

        None
    }

    ///
    /// Handles the lasso tool's input
    ///
    pub async fn handle_input<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, model: Arc<FloModel<Anim>>) {
        use self::ToolInput::*;

        let mut input       = input;
        let mut actions     = actions;
        let selection_model = model.selection();

        while let Some(input_event) = input.next().await {
            // Main input loop
            match input_event {
                Deselect        => { break; }
                Select          => { }
                Data(_)         => { }
                PaintDevice(_)  => { }

                Paint(painting) => {
                    if painting.action == PaintAction::Start {
                        let (x, y) = painting.location;

                        if selection_model.point_in_selection_path(x as f64, y as f64) {
                            // Clicking inside the path: drag the selection
                            // TODO: only if nothing is selected yet (if something is selected the path is already cut)
                            selection_model.cut_selection();

                            // Drag the selection
                            let translate = super::select::Select::drag_selection(painting, &mut input, &mut actions, &*model, LAYER_PREVIEW).await;

                            if let Some((dx, dy)) = translate {
                                // Translate the selection via the drag result
                                let selected_elements = selection_model.selected_elements.get().iter().cloned().collect();
                                model.perform_edits(vec![
                                    AnimationEdit::Element(selected_elements, ElementEdit::Transform(vec![
                                        ElementTransform::SetAnchor(0.0, 0.0),
                                        ElementTransform::MoveTo(dx as f64, dy as f64)
                                    ]))
                                ]);

                                // TODO: translate the selection path too
                            }
                        } else {
                            // Clicking outside of the path: create a new selection

                            // Clear the existing selected path
                            selection_model.selected_path.set(None);
                            selection_model.clear_selection();

                            // Select an area
                            let new_selection = Self::select_area(painting, &mut input, &mut actions).await;

                            // Set as the selected path
                            let new_selection_path = new_selection.map(|selection| Arc::new(Self::path_for_path(&selection)));
                            selection_model.selected_path.set(new_selection_path);
                        }
                    }
                }
            }
        }
    }

    ///
    /// Runs the lasso tool
    ///
    pub fn run<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, model: Arc<FloModel<Anim>>) -> impl Future<Output=()>+Send {
        async move {
            // Task that renders the selection path whenever it changes
            let render_selection_path   = Self::render_selection_path(BindRef::from(&model.selection().selected_path), actions.clone(), LAYER_SELECTION);

            // Task to handle the input from the user
            let handle_input            = Self::handle_input(input, actions, Arc::clone(&model));

            // Finish when either of the futures finish
            future::select_all(vec![render_selection_path.boxed(), handle_input.boxed()]).await;
        }
    }
}

impl<Anim: 'static+EditableAnimation> Tool<Anim> for Lasso {
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
            future: Mutex::new(ToolFuture::new(move |input, actions| { Self::run(input, actions, Arc::clone(&flo_model)) }))
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
    fn actions_for_input<'a>(&'a self, _flo_model: Arc<FloModel<Anim>>, tool_model: &Self::Model, _data: Option<Arc<Self::ToolData>>, input: Box<dyn 'a+Iterator<Item=ToolInput<Self::ToolData>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<Self::ToolData>>> {
        Box::new(tool_model.future.lock().unwrap().actions_for_input(input).into_iter())
    }
}

