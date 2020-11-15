use super::tool_input::*;
use super::tool_action::*;
use super::tool_future_streams::*;
use crate::model::*;

use flo_animation::*;

use futures::prelude::*;
use futures::stream;
use futures::stream::{BoxStream};

use std::iter;
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
    tool_input: Option<Arc<Mutex<ToolInputStreamCore<()>>>>
}

impl<CreateFutureFn, FutureResult> ToolFuture<CreateFutureFn>
where   CreateFutureFn: Fn(BoxStream<'static, ToolInput<()>>, ToolActionPublisher<()>) -> FutureResult + Send+Sync+'static,
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
            tool_input:     None
        }
    }

    ///
    /// Returns a stream of tool actions that result from changes to the model
    ///
    pub fn actions_for_model<ToolModel, Anim>(&mut self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &ToolModel) -> BoxStream<'static, ToolAction<()>> 
    where Anim: 'static+Animation+EditableAnimation {
        // TODO: when the returned stream is dropped, close the input stream

        // Close any existing input stream, stopping the future from running
        if let Some(tool_input) = self.tool_input.take() {
            close_tool_input(&tool_input);
            self.tool_input = None;
        }

        // Create a new future
        let (tool_input, tool_input_core) = create_tool_stream();
        self.tool_input         = Some(tool_input_core);

        let action_publisher    = create_tool_action_publisher();

        let new_future          = (self.create_future)(tool_input.boxed(), action_publisher);

        // TODO: return an action stream (which polls the future above)
        Box::pin(stream::empty())
    }

    ///
    /// Converts a set of tool inputs into the corresponding actions that should be performed
    ///
    pub fn actions_for_input<'a, Anim>(&'a self, flo_model: Arc<FloModel<Anim>>, data: Option<Arc<()>>, input: Box<dyn 'a+Iterator<Item=ToolInput<()>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<()>>>
    where Anim: 'static+Animation+EditableAnimation {
        // Send the input to the future if there is one. This can run the action stream as a side-effect
        self.tool_input.as_ref().map(|tool_input| send_tool_input(tool_input, input));

        // TODO: return any pending actions from the future immediately
        Box::new(iter::empty())
    }
}
