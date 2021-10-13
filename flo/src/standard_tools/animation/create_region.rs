use crate::tools::*;
use crate::model::*;
use super::super::lasso::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;
use flo_curves::bezier::*;
use flo_curves::bezier::path::{path_remove_interior_points};
use flo_canvas_animation::description::*;

use futures::prelude::*;
use futures::stream::{BoxStream};

use std::sync::*;

/// The region that the user is drawing as a new animation region
const LAYER_SELECTION: LayerId = LayerId(0);

///
/// The model for the CreateAnimationRegion tool
///
pub struct CreateAnimationRegionModel {
    /// The future that is running the create animation region tool at the moment
    future: Mutex<ToolFuture>
}

///
/// Tool that lets the user create and alter animation regions
///
pub struct CreateAnimationRegion {

}

#[inline] fn pathpoint_to_point2d(path_point: &PathPoint) -> Point2D {
    Point2D(path_point.position.0, path_point.position.1)
}

impl CreateAnimationRegion {
    ///
    /// Creates a new CreateAnimationRegion tool
    ///
    pub fn new() -> CreateAnimationRegion {
        CreateAnimationRegion { }
    }

    ///
    /// Allows the user to draw a new animation region path
    ///
    pub async fn draw_region(initial_event: Painting, input: &mut ToolInputStream<()>, actions: &mut ToolActionPublisher<()>) -> Option<Vec<BezierPath>> {
        // Use the same selection style as the freehand lasso tool
        let lasso_path = Lasso::select_area_freehand(initial_event, input, actions).await?;

        // Convert the path to a bezier path as used by the animation canvas
        if lasso_path.len() < 1 { return None; }

        let start_point = pathpoint_to_point2d(&lasso_path[0].start_point());
        let path        = lasso_path.into_iter()
            .map(|curve| {
                let (cp1, cp2)  = curve.control_points();
                let end_point   = curve.end_point();

                let cp1         = pathpoint_to_point2d(&cp1);
                let cp2         = pathpoint_to_point2d(&cp2);
                let end_point   = pathpoint_to_point2d(&end_point);

                BezierPoint(cp1, cp2, end_point)
            })
            .collect();

        // Remove interior points
        let path    = BezierPath(start_point, path);
        let path    = path_remove_interior_points(&vec![path], 0.01);

        Some(path)
    }

    ///
    /// Adds a new animation region to the animation
    ///
    fn add_new_region<Anim: 'static+EditableAnimation>(new_region: Vec<BezierPath>, actions: &mut ToolActionPublisher<()>, flo_model: &Arc<FloModel<Anim>>) {
        // Fetch the selected layer and the active keyframe
        let selected_layer  = flo_model.timeline().selected_layer.get();
        let keyframe_time   = flo_model.frame().keyframe_time.get();

        // If there's a keyframe and a layer, add a new animation region for that keyframe
        if let Some((selected_layer, keyframe_time)) = selected_layer.and_then(|layer| keyframe_time.map(move |time| (layer, time))) {
            // The region is initially an empty description with just this path
            let empty_region    = RegionDescription(new_region, EffectDescription::Sequence(vec![EffectDescription::FrameByFrameReplaceWhole]));

            // Generate the create region editing operation
            let create_region   = LayerEdit::CreateAnimation(keyframe_time, ElementId::Unassigned, empty_region);
            let create_region   = AnimationEdit::Layer(selected_layer, create_region);

            // Send as a tool action
            actions.send_actions(vec![ToolAction::EditAnimation(Arc::new(vec![create_region]))]);
        }
    }

    ///
    /// Runs the main input handling loop for this animation region
    ///
    async fn handle_input<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>) {
        use self::ToolInput::*;

        let mut input       = input;
        let mut actions     = actions;

        while let Some(input_event) = input.next().await {
            match input_event {
                Paint(painting) => {
                    if painting.action == PaintAction::Start && painting.modifier_keys == vec![] {
                        // Create a new animation region
                        let new_region = Self::draw_region(painting, &mut input, &mut actions).await;

                        if let Some(new_region) = new_region {
                            // Add as a new region
                            Self::add_new_region(new_region, &mut actions, &flo_model);
                        }
                    }
                }

                _ => { }
            }
        }
    }

    ///
    /// Runs the create animation region tool
    ///
    fn run<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>) -> impl Future<Output=()>+Send {
        async move {
            // The input handler loop tracks user mouse clicks and other such operations
            let handle_input = Self::handle_input(input, actions, Arc::clone(&flo_model));

            handle_input.await
        }
    }
}


impl<Anim: 'static+EditableAnimation> Tool<Anim> for CreateAnimationRegion {
    ///
    /// Represents data for the tool at a point in time (typically a snapshot
    /// of the model)
    ///
    type ToolData = ();

    ///
    /// The type of the model used by the UI elements of this tool
    ///
    type Model = CreateAnimationRegionModel;

    ///
    /// Retrieves the name of this tool
    ///
    fn tool_name(&self) -> String { String::from("Create animation region") }

    ///
    /// Retrieves the image that represents this tool in the toolbar
    ///
    fn image(&self) -> Option<Image> { 
        Some(svg_static(include_bytes!("../../../svg/tools/animation_region.svg")))
    }

    ///
    /// Creates a new instance of the UI model for this tool
    ///
    fn create_model(&self, flo_model: Arc<FloModel<Anim>>) -> Self::Model {
        CreateAnimationRegionModel {
            future:         Mutex::new(ToolFuture::new(move |input, actions| { Self::run(input, actions, Arc::clone(&flo_model)) })),
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
