use crate::menu::*;
use crate::tools::*;
use crate::model::*;

use flo_ui::*;
use flo_animation::*;

use futures::prelude::*;
use futures::stream::{BoxStream};

use std::sync::*;

///
/// The model for the Adjust tool
///
pub struct AdjustModel {
    future: Mutex<ToolFuture>
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
    /// The main input loop for the adjust tool
    ///
    pub fn handle_input<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>) -> impl Future<Output=()>+Send {
        async move {
            let mut input = input;

            while let Some(_input_event) = input.next().await {
            }
        }
    }

    ///
    /// Runs the adjust tool
    ///
    pub fn run<Anim: 'static+EditableAnimation>(input: ToolInputStream<()>, actions: ToolActionPublisher<()>, flo_model: Arc<FloModel<Anim>>) -> impl Future<Output=()>+Send {
        async move {
            // Task that renders the selection path whenever it changes
            // let render_selection_path   = Self::render_selection_path(BindRef::from(&flo_model.selection().selected_path), actions.clone(), LAYER_SELECTION);

            // Task to handle the input from the user
            let handle_input            = Self::handle_input(input, actions, Arc::clone(&flo_model));

            handle_input.await;

            // Finish when either of the futures finish
            //future::select_all(vec![render_selection_path.boxed(), handle_input.boxed()]).await;
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
