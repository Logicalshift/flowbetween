use super::tool_action::*;
use super::tool_input::*;
use super::super::model::*;

use animation::*;

use futures::*;
use futures::stream;

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
    /// Creates a new instance of the menu model for this tool
    /// 
    fn create_model(&self) -> Self::Model;

    ///
    /// Retrieves the menu controller to use for adjusting this tool
    /// 
    fn menu_controller_name(&self) -> String { "".to_string() }

    ///
    /// Returns a stream of tool actions that result from changes to the model
    /// 
    fn actions_for_model(&self, _model: Arc<FloModel<Anim>>) -> Box<Stream<Item=ToolAction<Self::ToolData>, Error=()>+Send> {
        Box::new(stream::empty())
    }

    ///
    /// Converts a set of tool inputs into the corresponding actions that should be performed
    /// 
    fn actions_for_input<'a>(&'a self, data: Option<Arc<Self::ToolData>>, input: Box<'a+Iterator<Item=ToolInput<Self::ToolData>>>) -> Box<'a+Iterator<Item=ToolAction<Self::ToolData>>>;
}

///
/// Equality so that tool objects can be referred to in bindings
/// 
impl<ToolData: Send+'static, Model: Send+Sync+'static, Anim: Animation> PartialEq for Tool<Anim, ToolData=ToolData, Model=Model> {
    fn eq(&self, other: &Tool<Anim, ToolData=ToolData, Model=Model>) -> bool {
        self.tool_name() == other.tool_name()
    }
}
