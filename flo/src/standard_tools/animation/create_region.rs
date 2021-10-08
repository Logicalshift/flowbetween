use crate::tools::*;
use crate::model::*;

use flo_ui::*;
use flo_animation::*;

use futures::stream;
use futures::stream::{BoxStream};

use std::iter;
use std::sync::*;

///
/// Tool that lets the user create and alter animation regions
///
pub struct CreateAnimationRegion {

}

impl CreateAnimationRegion {
    pub fn new() -> CreateAnimationRegion {
        CreateAnimationRegion { }
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
    type Model = ();

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
        ()
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
    fn actions_for_model(&self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &Self::Model) -> BoxStream<'static, ToolAction<Self::ToolData>> {
        Box::pin(stream::empty())
    }

    ///
    /// Converts a set of tool inputs into the corresponding actions that should be performed
    ///
    fn actions_for_input<'a>(&'a self, flo_model: Arc<FloModel<Anim>>, tool_model: &Self::Model, data: Option<Arc<Self::ToolData>>, input: Box<dyn 'a+Iterator<Item=ToolInput<Self::ToolData>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<Self::ToolData>>> {
        Box::new(iter::empty())
    }
}
