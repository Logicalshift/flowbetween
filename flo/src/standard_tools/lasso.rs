use crate::menu::*;
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
    /// The future that is running the Lasso tool at the moment
    future: Mutex<ToolFuture>,

    /// The bindings for the lasso tool
    tool_bindings: LassoBindings
}

///
/// The bindings for the lasso tool model
///
#[derive(Clone)]
pub struct LassoBindings {
    lasso_mode:     Binding<LassoMode>,
    lasso_shape:    Binding<LassoShape>
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
    fn drawing_for_path(path: &Option<Path>) -> Vec<Draw> {
        // Result is a drawing
        let mut path_drawing = vec![];

        // Start by clearing the preview layer
        path_drawing.layer(LAYER_PREVIEW);
        path_drawing.clear_layer();

        if let Some(path) = path {
            // Set up to render the path
            path_drawing.new_path();
            path_drawing.extend(path.to_drawing());
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
    /// Returns how a preview path should be rendered
    ///
    fn drawing_for_curves(path: &Option<Vec<Curve<PathPoint>>>) -> Vec<Draw> {
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
    /// Returns the rendering operations for a selected path
    ///
    pub fn drawing_for_selection_path(selected_path: &Path) -> Vec<Draw> {
        let mut draw_selected_path = vec![];

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

        draw_selected_path
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
                draw_selected_path.extend(Self::drawing_for_selection_path(&*selected_path));
            }

            // Publish an action to draw the path
            actions.send_actions(vec![
                ToolAction::Overlay(OverlayAction::Draw(draw_selected_path))
            ]);
        }
    }

    ///
    /// After the user starts drawing, selects an area on the canvas as a freehand path
    ///
    pub async fn select_area_freehand(initial_event: Painting, input: &mut ToolInputStream<()>, actions: &mut ToolActionPublisher<()>) -> Option<Vec<Curve<PathPoint>>> {
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
                    let select_drawing  = Self::drawing_for_curves(&select_path);
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
    /// Returns the path for a rectangle with an initial and a final point
    ///
    fn rectangle_path(initial_point: (f32, f32), end_point: (f32, f32)) -> Option<Path> {
        let (x1, y1)            = initial_point;
        let (x2, y2)            = end_point;

        // Not a rectangle if the start and end points are too close together
        let distance_squared    = (x1-x2)*(x1-x2) + (y1-y2)*(y1-y2);
        if distance_squared <= 2.0 {
            None
        } else {
            Some(Path::from_elements(vec![
                PathComponent::Move(PathPoint::new(x1, y1)),
                PathComponent::Line(PathPoint::new(x2, y1)),
                PathComponent::Line(PathPoint::new(x2, y2)),
                PathComponent::Line(PathPoint::new(x1, y2)),
                PathComponent::Line(PathPoint::new(x1, y1)),
                PathComponent::Close
            ]))
        }
    }

    ///
    /// After the user starts drawing, selects an area on the canvas as a rectangular path
    ///
    pub async fn select_area_rectangle(initial_event: Painting, input: &mut ToolInputStream<()>, actions: &mut ToolActionPublisher<()>) -> Option<Path> {
        use self::ToolInput::*;

        // Start with a point that's just at the initial location
        let initial_point = initial_event.location;
        let mut end_point = initial_point;

        // Read input until the user releases the mouse pointer
        while let Some(input) = input.next().await {
            match input {
                Paint(next_point) => {
                    // Only track events corresponding to the same pointer device as the initial action
                    if next_point.pointer_id != initial_event.pointer_id {
                        continue;
                    }

                    // Stop if the action is cancelled
                    if next_point.action == PaintAction::Cancel {
                        actions.send_actions(vec![
                            ToolAction::Overlay(OverlayAction::Draw(vec![Draw::Layer(LAYER_PREVIEW), Draw::ClearLayer]))
                        ]);
                        return None;
                    }

                    // Change the end point of the rectangle
                    end_point           = next_point.location;

                    // Create a rectangular path
                    let select_path     = Self::rectangle_path(initial_point, end_point);

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
    pub async fn handle_input<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>, tool_bindings: LassoBindings) {
        use self::ToolInput::*;

        let mut input       = input;
        let mut actions     = actions;
        let selection_model = flo_model.selection();

        while let Some(input_event) = input.next().await {
            // Main input loop
            match input_event {
                Deselect        => { break; }
                Select          => { }
                Data(_)         => { }
                PaintDevice(_)  => { }

                Paint(painting) => {
                    if painting.action == PaintAction::Start {
                        // Fetch the state at the start of the paint action
                        let (x, y)          = painting.location;
                        let existing_path   = selection_model.selected_path.get();
                        let mode            = tool_bindings.lasso_mode.get();
                        let mode            = if existing_path.is_none() { LassoMode::Select } else { mode };

                        if mode == LassoMode::Select && selection_model.point_in_selection_path(x as f64, y as f64) {
                            // Clicking inside the path: drag the selection
                            if selection_model.selected_elements.get().len() == 0 {
                                // (If we've generated the selection, then don't cut again)
                                selection_model.cut_selection();
                            }

                            // Drag the selection
                            let translate = super::select::Select::drag_selection(painting, &mut input, &mut actions, &*flo_model, LAYER_PREVIEW).await;

                            if let Some((dx, dy)) = translate {
                                // Translate the selection via the drag result
                                let selected_elements = selection_model.selected_elements.get().iter().cloned().collect();
                                flo_model.perform_edits(vec![
                                    AnimationEdit::Element(selected_elements, ElementEdit::Transform(vec![
                                        ElementTransform::SetAnchor(0.0, 0.0),
                                        ElementTransform::MoveTo(dx as f64, dy as f64)
                                    ]))
                                ]);

                                // Translate the selection path too
                                let translated_path = existing_path
                                    .map(|path| {
                                        Path::from_elements(path.elements()
                                            .map(|component| component.translate(dx as f64, dy as f64)))
                                    }).map(|path| Arc::new(path));
                                selection_model.selected_path.set(translated_path);

                                // Invalidate the canvas to show the updated view
                                flo_model.timeline().invalidate_canvas();
                            }
                        } else {
                            // Clicking outside of the path: create a new selection

                            if mode == LassoMode::Select {
                                // Clear the existing selected path
                                selection_model.selected_path.set(None);
                                selection_model.clear_selection();
                            }

                            // Select an area
                            let new_selection_path = match tool_bindings.lasso_shape.get() {
                                LassoShape::Freehand    => Self::select_area_freehand(painting, &mut input, &mut actions).await.map(|selection| Arc::new(Self::path_for_path(&selection))),
                                LassoShape::Rectangle   => Self::select_area_rectangle(painting, &mut input, &mut actions).await.map(|selection| Arc::new(selection)),
                                LassoShape::Ellipse     => Self::select_area_rectangle(painting, &mut input, &mut actions).await.map(|selection| Arc::new(selection)),
                            };

                            // Action depends on the lasso mode
                            match mode {
                                LassoMode::Select => {
                                    // Set as the selected path
                                    selection_model.selected_path.set(new_selection_path);
                                },

                                LassoMode::Add => {
                                    let new_path = existing_path
                                        .map(|existing_path| existing_path.to_subpaths())
                                        .and_then(|existing_path| new_selection_path.map(move |selection_path| (existing_path, selection_path)))
                                        .map(|(existing_path, selection_path)| path_add(&existing_path, &vec![&*selection_path], 0.01))
                                        .map(|combined_path| Arc::new(Path::from_paths(combined_path.iter())));
                                    selection_model.selected_path.set(new_path);
                                },

                                LassoMode::Subtract => {
                                    let new_path = existing_path
                                        .map(|existing_path| existing_path.to_subpaths())
                                        .and_then(|existing_path| new_selection_path.map(move |selection_path| (existing_path, selection_path)))
                                        .map(|(existing_path, selection_path)| path_sub(&existing_path, &vec![&*selection_path], 0.01))
                                        .map(|combined_path| Arc::new(Path::from_paths(combined_path.iter())));
                                    selection_model.selected_path.set(new_path);
                                },

                                LassoMode::Intersect => {
                                    let new_path = existing_path
                                        .map(|existing_path| existing_path.to_subpaths())
                                        .and_then(|existing_path| new_selection_path.map(move |selection_path| (existing_path, selection_path)))
                                        .map(|(existing_path, selection_path)| path_intersect(&existing_path, &vec![&*selection_path], 0.01))
                                        .map(|combined_path| Arc::new(Path::from_paths(combined_path.iter())));
                                    selection_model.selected_path.set(new_path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    ///
    /// Runs the lasso tool
    ///
    pub fn run<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>, tool_bindings: LassoBindings) -> impl Future<Output=()>+Send {
        async move {
            // Task that renders the selection path whenever it changes
            let render_selection_path   = Self::render_selection_path(BindRef::from(&flo_model.selection().selected_path), actions.clone(), LAYER_SELECTION);

            // Task to handle the input from the user
            let handle_input            = Self::handle_input(input, actions, Arc::clone(&flo_model), tool_bindings);

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
        let tool_bindings       = LassoBindings {
            lasso_mode:     Binding::new(LassoMode::Select),
            lasso_shape:    Binding::new(LassoShape::Freehand)
        };
        let run_tool_bindings   = tool_bindings.clone();

        LassoModel {
            future:         Mutex::new(ToolFuture::new(move |input, actions| { Self::run(input, actions, Arc::clone(&flo_model), run_tool_bindings.clone()) })),
            tool_bindings:  tool_bindings
        }
    }

    ///
    /// Creates the menu controller for this tool (or None if this tool has no menu controller)
    ///
    fn create_menu_controller(&self, flo_model: Arc<FloModel<Anim>>, tool_model: &Self::Model) -> Option<Arc<dyn Controller>> {
        // Fetch the model
        let lasso_mode      = &tool_model.tool_bindings.lasso_mode;
        let lasso_shape     = &tool_model.tool_bindings.lasso_shape;
        let selected_path   = flo_model.selection().selected_path.clone();

        Some(Arc::new(LassoMenuController::new(lasso_mode, lasso_shape, &selected_path)))
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

