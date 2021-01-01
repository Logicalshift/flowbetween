use crate::tools::*;
use crate::model::*;

use flo_ui::*;
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
pub struct Shape { }

impl Shape {
    ///
    /// Creates the Shape tool
    ///
    pub fn new() -> Shape {
        Shape { 
        }
    }

    ///
    /// The main input loop for the shape tool
    ///
    pub fn handle_input<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>) -> impl Future<Output=()>+Send {
        async move {
            let mut input = input;

            while let Some(_input_event) = input.next().await {
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

impl<Anim: 'static+EditableAnimation> Tool<Anim> for Shape {
    type ToolData   = ();
    type Model      = ShapeModel;

    fn tool_name(&self) -> String { "".to_string() }

    fn image(&self) -> Option<Image> { Some(svg_static(include_bytes!("../../svg/tools/adjust.svg"))) }

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