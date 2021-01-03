use crate::tools::*;
use crate::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use futures::prelude::*;
use futures::stream::{BoxStream};

use std::sync::*;

///
/// The model for the Shape tool
///
pub struct ShapeModel {
    future: Mutex<ToolFuture>
}

///
/// The Shape tool, which creates shapes
///
pub struct ShapeTool {
    name:                   String,
    icon:                   Image,
    create_shape_element:   Arc<dyn Fn((f64, f64), (f64, f64)) -> ShapeElement+Send+Sync>
}

impl ShapeTool {
    ///
    /// Creates the Shape tool
    ///
    pub fn new<CreateShapeFn: Fn((f64, f64), (f64, f64)) -> ShapeElement+Send+Sync+'static>(name: &str, icon: Image, create_shape_fn: CreateShapeFn) -> ShapeTool {
        let name = name.to_string();

        ShapeTool { 
            name:                   name,
            icon:                   icon,
            create_shape_element:   Arc::new(create_shape_fn)
        }
    }

    ///
    /// Creates the rectangle shape tool
    ///
    pub fn rectangle() -> ShapeTool {
        Self::new("Shape-rectangle", svg_static(include_bytes!("../../svg/tools/shape_rectangle.svg")), |center, point| ShapeElement::new(ElementId::Unassigned, 0.5, Shape::Rectangle { center, point }))
    }

    ///
    /// Creates the ellipse shape tool
    ///
    pub fn ellipse() -> ShapeTool {
        Self::new("Shape-ellipse", svg_static(include_bytes!("../../svg/tools/shape_ellipse.svg")), |center, point| ShapeElement::new(ElementId::Unassigned, 0.5, Shape::Circle { center, point }))
    }

    ///
    /// Creates the polygon shape tool
    ///
    pub fn polygon() -> ShapeTool {
        // TODO: support configuring the number of sides
        Self::new("Shape-polygon", svg_static(include_bytes!("../../svg/tools/shape_polygon.svg")), |center, point| ShapeElement::new(ElementId::Unassigned, 0.5, Shape::Polygon { sides: 3, center, point }))
    }

    ///
    /// The use has started drawing a new shape
    ///
    async fn drag_new_shape<Anim: 'static+EditableAnimation>(initial_action: Painting, input: &mut ToolInputStream<()>, actions: &ToolActionPublisher<()>, flo_model: &Arc<FloModel<Anim>>, create_shape_element: &Arc<dyn Fn((f64, f64), (f64, f64)) -> ShapeElement+Send+Sync>) {
        // Get the current settings for the animation
        let layer   = flo_model.timeline().selected_layer.get();
        let layer   = if let Some(layer) = layer { layer } else { return; };
        let when    = flo_model.timeline().current_time.get();

        // TODO: read brush properties from the current brush
        // Set up the brush preview for the shape
        actions.send_actions(vec![
            ToolAction::BrushPreview(BrushPreviewAction::Clear),
            ToolAction::BrushPreview(BrushPreviewAction::Layer(layer)),
            ToolAction::BrushPreview(BrushPreviewAction::Clear),
            ToolAction::BrushPreview(BrushPreviewAction::Layer(layer)),
            ToolAction::BrushPreview(BrushPreviewAction::BrushDefinition(BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw)),
            ToolAction::BrushPreview(BrushPreviewAction::BrushProperties(BrushProperties::new()))
        ]);

        while let Some(input_event) = input.next().await {
            match input_event {
                ToolInput::Paint(painting) => {
                    // Input from other pointing devices cancels the action
                    if painting.pointer_id != initial_action.pointer_id {
                        break;
                    }

                    // Construct the shape for this painting event
                    let center          = (initial_action.location.0 as f64, initial_action.location.1 as f64);
                    let point           = (painting.location.0 as f64, painting.location.1 as f64);

                    let shape_element   = (create_shape_element)(center, point);

                    match painting.action {
                        PaintAction::Continue |
                        PaintAction::Prediction => {
                            // Draw the shape as a brush preview
                            let brush_points = shape_element.brush_points();

                            actions.send_actions(vec![ToolAction::BrushPreview(BrushPreviewAction::SetBrushPoints(Arc::new(brush_points)))]);
                        }

                        PaintAction::Finish => {
                            // TODO: set the brush properties

                            // Reset the brush preview and create a new keyframe if needed
                            actions.send_actions(vec![
                                ToolAction::CreateKeyFrameForDrawing,
                                ToolAction::BrushPreview(BrushPreviewAction::Clear),
                                ToolAction::EditAnimation(Arc::new(vec![
                                    AnimationEdit::Layer(layer, LayerEdit::Paint(when, PaintEdit::CreateShape(ElementId::Unassigned, shape_element.width() as f32, shape_element.shape())))
                                ])),
                                ToolAction::InvalidateFrame
                            ]);
                            break;
                        }

                        PaintAction::Cancel |
                        PaintAction::Start => {
                            // Just stop receiving events (start works the same as 'cancel' as it indicates we somehow got out of sync with the state of the button)
                            break;
                        }
                    }
                }

                _ => { }
            }
        }

        // Clear the brush preview
        actions.send_actions(vec![ToolAction::BrushPreview(BrushPreviewAction::Clear)]);
    }

    ///
    /// The main input loop for the shape tool
    ///
    pub fn handle_input<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>, create_shape_element: Arc<dyn Fn((f64, f64), (f64, f64)) -> ShapeElement+Send+Sync>) -> impl Future<Output=()>+Send {
        async move {
            let mut input = input;

            while let Some(input_event) = input.next().await {
                match input_event {
                    ToolInput::Paint(painting) => {
                        if painting.action == PaintAction::Start {
                            actions.send_actions(vec![ToolAction::BrushPreview(BrushPreviewAction::Clear)]);
                            Self::drag_new_shape(painting, &mut input, &actions, &flo_model, &create_shape_element).await;
                        }
                    }

                    _ => { }
                }
            }
        }
    }

    ///
    /// Runs the shape tool
    ///
    pub fn run<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>, create_shape_element: Arc<dyn Fn((f64, f64), (f64, f64)) -> ShapeElement+Send+Sync>) -> impl Future<Output=()>+Send {
        async move {
            // Task to handle the input from the user
            let handle_input            = Self::handle_input(input, actions, Arc::clone(&flo_model), create_shape_element);

            handle_input.await;
        }
    }
}

impl<Anim: 'static+EditableAnimation> Tool<Anim> for ShapeTool {
    type ToolData   = ();
    type Model      = ShapeModel;

    fn tool_name(&self) -> String {
        self.name.clone()
    }

    fn image(&self) -> Option<Image> { Some(self.icon.clone()) }

    fn create_model(&self, flo_model: Arc<FloModel<Anim>>) -> ShapeModel { 
        let create_shape_element = Arc::clone(&self.create_shape_element);

        ShapeModel {
            future: Mutex::new(ToolFuture::new(move |input, actions| { Self::run(input, actions, Arc::clone(&flo_model), Arc::clone(&create_shape_element)) }))
        }
    }

    fn create_menu_controller(&self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &ShapeModel) -> Option<Arc<dyn Controller>> {
        None
    }

    ///
    /// Returns a stream containing the actions for the view and tool model for the select tool
    ///
    fn actions_for_model(&self, _flo_model: Arc<FloModel<Anim>>, tool_model: &ShapeModel) -> BoxStream<'static, ToolAction<()>> {
        tool_model.future.lock().unwrap().actions_for_model()
    }

    fn actions_for_input<'a>(&'a self, _flo_model: Arc<FloModel<Anim>>, tool_model: &ShapeModel, _data: Option<Arc<()>>, input: Box<dyn 'a+Iterator<Item=ToolInput<()>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<()>>> {
        Box::new(tool_model.future.lock().unwrap().actions_for_input(input).into_iter())
    }
}