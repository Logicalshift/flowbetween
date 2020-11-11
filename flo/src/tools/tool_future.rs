use super::tool_input::*;
use super::tool_action::*;
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
/// implementing a tool model and 
///
pub struct ToolFuture<CreateFutureFn> {
    create_future: CreateFutureFn
}

impl<CreateFutureFn, FutureResult> ToolFuture<CreateFutureFn>
where   CreateFutureFn: Fn(BoxStream<'static, ToolInput<()>>, Box<dyn Fn(ToolAction<()>) -> ()>) -> FutureResult + Send+Sync+'static,
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
            create_future
        }
    }

    ///
    /// Returns a stream of tool actions that result from changes to the model
    ///
    pub fn actions_for_model<ToolModel, Anim>(&self, _flo_model: Arc<FloModel<Anim>>, _tool_model: &ToolModel) -> BoxStream<'static, ToolAction<()>> 
    where Anim: 'static+Animation+EditableAnimation {
        Box::pin(stream::empty())
    }

    ///
    /// Converts a set of tool inputs into the corresponding actions that should be performed
    ///
    pub fn actions_for_input<'a, Anim>(&'a self, flo_model: Arc<FloModel<Anim>>, data: Option<Arc<()>>, input: Box<dyn 'a+Iterator<Item=ToolInput<()>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<()>>>
    where Anim: 'static+Animation+EditableAnimation {
        Box::new(iter::empty())
    }
}
