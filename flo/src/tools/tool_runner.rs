use super::tool_action::*;
use super::generic_tool::*;
use super::super::viewmodel::*;

use animation::*;

use futures::*;
use futures::executor;
use futures::executor::Spawn;
use std::sync::*;

///
/// Runs the actions for a particular tool
/// 
pub struct ToolRunner<Anim: Animation> {
    /// The view model that is passed to the tools
    view_model: Arc<AnimationViewModel<Anim>>,

    /// The currently active tool
    current_tool: Option<Arc<FloTool<Anim>>>,

    /// Most recent tool data from the current tool
    tool_data: Option<GenericToolData>,

    /// The model actions specified by the current tool
    model_actions: Option<Spawn<Box<Stream<Item=ToolAction<GenericToolData>, Error=()>>>>
}

impl<Anim: Animation> ToolRunner<Anim> {
    ///
    /// Creates a new tool runner
    /// 
    pub fn new(view_model: &AnimationViewModel<Anim>) -> ToolRunner<Anim> {
        let view_model = Arc::new(view_model.clone());

        ToolRunner {
            view_model:     view_model,
            current_tool:   None,
            tool_data:      None,
            model_actions:  None
        }
    }

    ///
    /// Sets the tool that this will use to run its actions on
    /// 
    pub fn set_tool(&mut self, new_tool: &Arc<FloTool<Anim>>) {
        // Free the data for the current tool
        self.tool_data      = None;
        self.model_actions  = None;

        // Set the new tool
        let model_actions   = new_tool.actions_for_model(Arc::clone(&self.view_model));
        self.current_tool   = Some(Arc::clone(&new_tool));

        // Start the actions running for the new tool
        self.model_actions  = Some(executor::spawn(model_actions));
    }
}
