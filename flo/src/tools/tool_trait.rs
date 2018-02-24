use super::tool_action::*;
use super::tool_input::*;
use super::super::viewmodel::*;

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
pub trait Tool2<ToolData: 'static, Anim: Animation> {
    ///
    /// Retrieves the name of this tool
    /// 
    fn tool_name(&self) -> String;

    ///
    /// Retrieves the name of the image that is associated with this tool
    /// 
    fn image_name(&self) -> String;

    ///
    /// Retrieves the menu controller to use for adjusting this tool
    /// 
    fn menu_controller_name(&self) -> String { "".to_string() }

    ///
    /// Returns a stream of tool actions that result from changes to the model
    /// 
    fn actions_for_model(&self, _model: Arc<AnimationViewModel<Anim>>) -> Box<Stream<Item=ToolAction<ToolData>, Error=()>> {
        Box::new(stream::empty())
    }

    ///
    /// Converts a set of tool inputs into the corresponding actions that should be performed
    /// 
    fn actions_for_input<'b>(&self, data: Option<&'b ToolData>, input: Box<Iterator<Item=ToolInput<'b, ToolData>>>) -> Box<'b+Iterator<Item=ToolAction<ToolData>>>;
}
