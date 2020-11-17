use super::tool_input::*;
use super::tool_action::*;
use super::tool_future_streams::*;
use crate::model::*;

use flo_animation::*;

use futures::prelude::*;
use futures::stream::{BoxStream};

use std::sync::*;

///
/// Allows a tool to be implemented as a single future
///
/// This can be used as the tool model for tools that want to use an 'async' function to manage their state instead of
/// implementing a tool model and tool data structure and performing manual state transitions
///
pub struct ToolFuture<CreateFutureFn> {
    /// Function that creates a new future to run the tool
    create_future: CreateFutureFn,

    /// The active input stream core (or none if one doesn't exist)
    tool_input: Option<Arc<Mutex<ToolStreamCore<ToolInput<()>>>>>,

    /// The active action stream core (or none if one doesn't exist)
    tool_actions: Option<Arc<Mutex<ToolStreamCore<ToolAction<()>>>>>
}

impl<CreateFutureFn, FutureResult> ToolFuture<CreateFutureFn>
where   CreateFutureFn: Fn(ToolInputStream<()>, ToolActionPublisher<()>) -> FutureResult + Send+Sync+'static,
        FutureResult:   Future<Output=()> + Send+Sync+'static {
    ///
    /// Creates a new ToolFuture from a future factory function
    ///
    /// The 'Create' function takes two arguments: a list of tool inputs and a function to generate actions. Actions can be generated
    /// at any time, and will be returned via the model stream or the input stream depending on which one is evaluated first.
    ///
    /// The future starts executing when `actions_for_model()` is called, and will be cancelled by dropping if the output stream is
    /// closed. Only the output from the most recent future will be returned from the stream, in case two futures are running at one
    /// time for any reason. In particular, this means that any state that needs to preserved from one activation to another must be
    /// stored in either the tool model or the main FloModel.
    ///
    pub fn new(create_future: CreateFutureFn) -> ToolFuture<CreateFutureFn> {
        ToolFuture {
            create_future:  create_future,
            tool_input:     None,
            tool_actions:   None
        }
    }

    ///
    /// Returns a stream of tool actions that result from changes to the model
    ///
    pub fn actions_for_model<ToolModel, Anim>(&mut self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &ToolModel) -> BoxStream<'static, ToolAction<()>> 
    where Anim: 'static+Animation+EditableAnimation {
        // Close any existing input stream, stopping the future from running
        if let Some(tool_input) = self.tool_input.take() {
            close_tool_stream(&tool_input);
            self.tool_input = None;
        }

        if let Some(tool_actions) = self.tool_actions.take() {
            close_tool_stream(&tool_actions);
            self.tool_actions = None;
        }

        // The tool input stream is used to send data from actions_for_input to the future
        let (tool_input, tool_input_core)   = create_tool_input_stream();

        // The action stream is returned from this function, and the publisher is used to send actions to the future
        let (action_stream, action_core)    = create_tool_action_stream(&tool_input_core);
        let action_publisher                = create_tool_action_publisher(&action_core);

        // Create the new future
        let new_future                      = (self.create_future)(tool_input, action_publisher);

        // The future is run as a side-effect of polling the stream
        let mut action_stream               = action_stream;
        action_stream.set_future(new_future);

        // Store the results in this structure
        self.tool_input                     = Some(tool_input_core);
        self.tool_actions                   = Some(action_core);

        // The action stream is the end result
        action_stream.boxed()
    }

    ///
    /// Converts a set of tool inputs into the corresponding actions that should be performed
    ///
    pub fn actions_for_input<'a, Anim>(&'a self, flo_model: Arc<FloModel<Anim>>, data: Option<Arc<()>>, input: Box<dyn 'a+Iterator<Item=ToolInput<()>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<()>>>
    where Anim: 'static+Animation+EditableAnimation {
        // Send the input to the future if there is one. This can run the action stream as a side-effect
        self.tool_input.as_ref().map(|tool_input| send_tool_stream(tool_input, input));

        // Return any pending actions from the future immediately
        self.tool_actions.as_ref()
            .map(|tool_actions| Box::new(drain_tool_stream(tool_actions).into_iter()))
            .unwrap_or_else(|| Box::new(vec![].into_iter()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use flo_animation::editor::*;
    use flo_animation::storage::*;

    use futures::executor;

    ///
    /// Creates an animation model to use in the tests
    ///
    fn create_model() -> Arc<FloModel<impl 'static+Animation+EditableAnimation>> {
        // Create an animation
        let in_memory_store = InMemoryStorage::new();
        let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());
        let model           = FloModel::new(animation);

        Arc::new(model)
    }

    #[test]
    fn generate_tool_actions() {
        executor::block_on(async move {
            // Create a new tool future that sends some actions
            let mut tool_future     = ToolFuture::new(|_input_stream, action_output| {
                async move {
                    action_output.send_actions(vec![ToolAction::InvalidateFrame, ToolAction::ClearSelection]);
                    action_output.send_actions(vec![ToolAction::Select(ElementId::Assigned(0)), ToolAction::Select(ElementId::Assigned(1))]);
                }
            });

            // Create the action stream
            let model               = create_model();
            let mut action_stream   = tool_future.actions_for_model(model, &());

            // Check that the actions arrive in the expected order
            assert!(action_stream.next().await == Some(ToolAction::InvalidateFrame));
            assert!(action_stream.next().await == Some(ToolAction::ClearSelection));
            assert!(action_stream.next().await == Some(ToolAction::Select(ElementId::Assigned(0))));
            assert!(action_stream.next().await == Some(ToolAction::Select(ElementId::Assigned(1))));

            // The stream should close once the future ends
            assert!(action_stream.next().await == None);
        });
    }
}
