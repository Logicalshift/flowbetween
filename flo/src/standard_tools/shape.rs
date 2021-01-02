use crate::tools::*;
use crate::model::*;

use flo_ui::*;
use flo_stream::*;
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
    name: String,
    icon: Image
}

impl ShapeTool {
    ///
    /// Creates the Shape tool
    ///
    pub fn new(name: &str, icon: Image) -> ShapeTool {
        let name = name.to_string();

        ShapeTool { 
            name,
            icon
        }
    }

    ///
    /// Creates a shape element
    ///
    fn create_shape_element(center: (f64, f64), point: (f64, f64)) -> ShapeElement {
        ShapeElement::new(ElementId::Unassigned, 0.5, Shape::Polygon { sides: 6, center, point })
    }

    ///
    /// The use has started drawing a new shape
    ///
    async fn drag_new_shape<Anim: 'static+EditableAnimation>(initial_action: Painting, input: &mut ToolInputStream<()>, actions: &ToolActionPublisher<()>, flo_model: &Arc<FloModel<Anim>>) {
        // Get the current settings for the animation
        let layer   = flo_model.timeline().selected_layer.get();
        let layer   = if let Some(layer) = layer { layer } else { return; };
        let when    = flo_model.timeline().current_time.get();

        // Set up the brush preview for the shape
        actions.send_actions(vec![
            ToolAction::BrushPreview(BrushPreviewAction::Clear),
            ToolAction::BrushPreview(BrushPreviewAction::Layer(layer)),
            ToolAction::BrushPreview(BrushPreviewAction::Clear),
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

                    let shape_element   = Self::create_shape_element(center, point);

                    match painting.action {
                        PaintAction::Continue |
                        PaintAction::Prediction => {
                            // Draw the shape as a brush preview
                            let brush_points = shape_element.brush_points();

                            actions.send_actions(vec![ToolAction::BrushPreview(BrushPreviewAction::SetBrushPoints(Arc::new(brush_points)))]);
                        }

                        PaintAction::Finish => {
                            // Commit the shape
                            flo_model.edit().publish(Arc::new(vec![
                                AnimationEdit::Layer(layer, LayerEdit::Paint(when, PaintEdit::CreateShape(ElementId::Unassigned, shape_element.width() as f32, shape_element.shape())))
                            ])).await;

                            // Reset the brush preview
                            actions.send_actions(vec![
                                ToolAction::BrushPreview(BrushPreviewAction::Clear),
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
    pub fn handle_input<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>) -> impl Future<Output=()>+Send {
        async move {
            let mut input = input;

            while let Some(input_event) = input.next().await {
                match input_event {
                    ToolInput::Paint(painting) => {
                        if painting.action == PaintAction::Start {
                            actions.send_actions(vec![ToolAction::BrushPreview(BrushPreviewAction::Clear)]);
                            Self::drag_new_shape(painting, &mut input, &actions, &flo_model).await;
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
    pub fn run<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>) -> impl Future<Output=()>+Send {
        async move {
            // Task to handle the input from the user
            let handle_input            = Self::handle_input(input, actions, Arc::clone(&flo_model));

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
        ShapeModel {
            future:         Mutex::new(ToolFuture::new(move |input, actions| { Self::run(input, actions, Arc::clone(&flo_model)) }))
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