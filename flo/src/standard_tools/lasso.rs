use crate::tools::*;
use crate::model::*;

use flo_ui::*;
use flo_animation::*;

use futures::prelude::*;
use futures::stream::{BoxStream};

use std::sync::*;

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
    /// Implementation of the logic for the lasso tool
    ///
    pub async fn run(input: ToolInputStream<()>, actions: ToolActionPublisher<()>) {
        let mut input = input;

        println!("New");

        while let Some(input) = input.next().await {
            println!("{:?}", input);
        }

        println!("Done");
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

