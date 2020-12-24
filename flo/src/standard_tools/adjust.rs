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
use std::sync::*;

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
    fn declare_bezier_point_sprite(sprite_id: SpriteId) -> Vec<Draw> {
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
        draw.fill_color(CP_BEZIER_OUTLINE);
        draw.fill();

        draw.new_path();
        draw.move_to(0.0, RADIUS);
        for point in 1..NUM_SIDES {
            let angle = (2.0*f32::consts::PI)*((point as f32)/(NUM_SIDES as f32));
            draw.line_to(angle.sin()*RADIUS, angle.cos()*RADIUS);
        }
        draw.close_path();
        draw.fill_color(CP_BEZIER);
        draw.fill();

        draw
    }

    ///
    /// The main input loop for the adjust tool
    ///
    fn handle_input<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>) -> impl Future<Output=()>+Send {
        async move {
            let mut input = input;

            while let Some(_input_event) = input.next().await {
            }
        }
    }

    ///
    /// Generates the selection preview drawing actions from a model
    ///
    fn generate_selection_preview<Anim: 'static+EditableAnimation>(flo_model: &FloModel<Anim>) -> Vec<Draw> {
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
    fn generate_control_points(control_points: &Vec<AdjustControlPoint>) -> Vec<Draw> {
        let mut drawing = vec![];

        for cp in control_points.into_iter().filter(|cp| !cp.control_point.is_control_point()) {
            // Draw a control point sprite
            let (x, y) = cp.control_point.position();

            drawing.sprite_transform(SpriteTransform::Identity);
            drawing.sprite_transform(SpriteTransform::Translate(x as f32, y as f32));
            drawing.draw_sprite(SPRITE_BEZIER_POINT);
        }

        drawing
    }

    ///
    /// Tracks the selection path and renders the control points and selection preview
    ///
    async fn render_selection_path<Anim: 'static+EditableAnimation>(actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>, control_points: BindRef<Arc<Vec<AdjustControlPoint>>>) {
        // Create a binding that tracks the rendering actions for the current selection
        let model               = flo_model.clone();
        let selection_preview   = computed(move || Self::generate_selection_preview(&*model));
        let model               = flo_model.clone();
        let cp_preview          = computed(move || Self::generate_control_points(&*control_points.get()));

        // Combine the two previews whenever the selection changes
        let preview             = computed(move || {
            let selection_preview   = selection_preview.get();
            let cp_preview          = cp_preview.get();

            let mut preview         = vec![Draw::Layer(LAYER_SELECTION)];
            preview.extend(selection_preview);
            preview.push(Draw::Layer(LAYER_PREVIEW));
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
            let model           = flo_model.clone();
            let control_points  = computed(move || Arc::new(Self::control_points_for_selection(&*model)));
            let control_points  = BindRef::from(control_points);

            // Declare the sprites for the adjust tool
            actions.send_actions(vec![ToolAction::Overlay(OverlayAction::Draw(Self::declare_bezier_point_sprite(SPRITE_BEZIER_POINT)))]);

            // Task that renders the selection path whenever it changes
            let render_selection_path   = Self::render_selection_path(actions.clone(), flo_model.clone(), control_points.clone());

            // Task to handle the input from the user
            let handle_input            = Self::handle_input(input, actions, Arc::clone(&flo_model));

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
