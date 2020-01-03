use super::tool_action::*;
use super::tool_input::*;
use super::super::model::*;

use flo_ui::*;
use flo_animation::*;

use futures::stream;
use futures::stream::{BoxStream};

use std::sync::*;

///
/// Trait implemented by something representing a tool
///
/// TODO: way for the tool to specify a controller and a model so that
/// it can store/update its state.
///
/// TODO: way for the tool to serialize its state to the animation
///
pub trait Tool<Anim: Animation> : Send+Sync {
    ///
    /// Represents data for the tool at a point in time (typically a snapshot
    /// of the model)
    ///
    type ToolData: Send+'static;

    ///
    /// The type of the model used by the UI elements of this tool
    ///
    type Model: Send+Sync+'static;

    ///
    /// Retrieves the name of this tool
    ///
    fn tool_name(&self) -> String;

    ///
    /// Retrieves the name of the image that is associated with this tool
    ///
    fn image_name(&self) -> String;

    ///
    /// Creates a new instance of the UI model for this tool
    ///
    fn create_model(&self, flo_model: Arc<FloModel<Anim>>) -> Self::Model;

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
    fn actions_for_input<'a>(&'a self, flo_model: Arc<FloModel<Anim>>, data: Option<Arc<Self::ToolData>>, input: Box<dyn 'a+Iterator<Item=ToolInput<Self::ToolData>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<Self::ToolData>>>;
}

///
/// Equality so that tool objects can be referred to in bindings
///
impl<ToolData: Send+'static, Model: Send+Sync+'static, Anim: Animation> PartialEq for dyn Tool<Anim, ToolData=ToolData, Model=Model> {
    fn eq(&self, other: &dyn Tool<Anim, ToolData=ToolData, Model=Model>) -> bool {
        self.tool_name() == other.tool_name()
    }
}
