use super::tool_input::*;
use super::tool_action::*;
use super::shared_future::*;
use super::tool_future_streams::*;
use crate::model::*;

use flo_animation::*;

use futures::prelude::*;
use futures::task::{Poll};
use futures::future::{BoxFuture};
use futures::stream::{BoxStream};

use std::sync::*;

///
/// Allows a tool to be implemented as a single future
///
/// This can be used as the tool model for tools that want to use an 'async' function to manage their state instead of
/// implementing a tool model and tool data structure and performing manual state transitions
///
pub struct ToolFuture {
    /// Function that creates a new future to run the tool
    create_future: Box<dyn Fn(ToolInputStream<()>, ToolActionPublisher<()>) -> BoxFuture<'static, ()> + Send+Sync>,

    /// The active input stream core (or none if one doesn't exist)
    tool_input: Option<Arc<Mutex<ToolStreamCore<ToolInput<()>>>>>,

    /// The active action stream core (or none if one doesn't exist)
    tool_actions: Option<Arc<Mutex<ToolStreamCore<ToolAction<()>>>>>,

    /// The active future, if there is one
    future: Option<SharedFuture<()>>
}

impl ToolFuture {
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
    pub fn new<CreateFutureFn, FutureResult>(create_future: CreateFutureFn) -> ToolFuture
    where   CreateFutureFn: Fn(ToolInputStream<()>, ToolActionPublisher<()>) -> FutureResult + Send+Sync+'static,
            FutureResult:   Future<Output=()> + Send+Sync+'static {
        ToolFuture {
            create_future:  Box::new(move |input_stream, action_publisher| (create_future)(input_stream, action_publisher).boxed()),
            tool_input:     None,
            tool_actions:   None,
            future:         None
        }
    }

    ///
    /// Returns a stream of tool actions that result from changes to the model
    ///
    pub fn actions_for_model(&mut self) -> BoxStream<'static, ToolAction<()>> {
        // Close any existing input stream, stopping the future from running
        self.future = None;

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
        let new_future                      = SharedFuture::new(new_future);
        self.future                         = Some(new_future.clone());

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
    pub fn actions_for_input<'a>(&'a self, input: Box<dyn 'a+Iterator<Item=ToolInput<()>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<()>>> {
        // Send the input to the future if there is one. This can run the action stream as a side-effect
        self.tool_input.as_ref().map(|tool_input| send_tool_stream(tool_input, input));

        // Give the future a chance to run
        if let Some(Poll::Ready(())) = self.future.as_ref().map(|future| future.check()) {
            // TODO: Shut the future down if it stops as a result of this request
        }

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
            let mut action_stream   = tool_future.actions_for_model();

            // Check that the actions arrive in the expected order
            assert!(action_stream.next().await == Some(ToolAction::InvalidateFrame));
            assert!(action_stream.next().await == Some(ToolAction::ClearSelection));
            assert!(action_stream.next().await == Some(ToolAction::Select(ElementId::Assigned(0))));
            assert!(action_stream.next().await == Some(ToolAction::Select(ElementId::Assigned(1))));

            // The stream should close once the future ends
            assert!(action_stream.next().await == None);
        });
    }

    #[test]
    fn tool_input_generates_actions_immediately() {
        executor::block_on(async move {
            // Create a new tool future that sends some actions
            let mut tool_future     = ToolFuture::new(|input_stream, action_output| {
                async move {
                    let mut input_stream    = input_stream;
                    let mut counter         = 0;

                    while let Some(action) = input_stream.next().await {
                        println!("Send action {:?}", counter);
                        action_output.send_actions(vec![ToolAction::Select(ElementId::Assigned(counter))]);
                        counter += 1;
                    }
                }
            });

            // Create the action stream, so the tool is running
            let model               = create_model();
            let _action_stream      = tool_future.actions_for_model();

            // When we send input requests, we should generate the actions immediately here (as the future awaits nothing before sending the output)
            assert!(tool_future.actions_for_input(Box::new(vec![ToolInput::Select].into_iter())).collect::<Vec<_>>() == vec![ToolAction::Select(ElementId::Assigned(0))]);
        });
    }
}
