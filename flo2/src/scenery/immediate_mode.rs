use flo_scene::*;

use futures::prelude::*;
use futures::future::{BoxFuture};

use std::sync::*;

///
/// Context for an immediate-mode subprogram
///
pub struct ImContext<TInput, TState> {
    /// Function that handles updating the state. Called when waiting for a new state, should await the future to retrieve the input for the current program
    update_state: Arc<dyn Send + Sync + for<'a> Fn(BoxFuture<'a, Option<Vec<TInput>>>, TState, SceneContext) -> BoxFuture<'a, Option<TState>>>,

    /// The input stream for the subprogram
    input_stream: stream::ReadyChunks<InputStream<TInput>>,

    /// The context for the subprogram
    context: SceneContext,
}

impl<TInput, TState> ImContext<TInput, TState>
where 
    TInput: SceneMessage,
    TState: Send,
{
    ///
    /// Performs any processing required and returns the next state once that's complete
    ///
    pub async fn next(&mut self, state: TState) -> Option<TState> {
        let input_stream    = &mut self.input_stream;
        let context         = self.context.clone();
        let update_state    = &mut self.update_state;

        let next_input = input_stream.next();

        update_state(next_input.boxed(), state, context).await
    }
}

///
/// An 'immediate mode' subprogram is one that runs in two phases: it updates some state in one phase and updates that
/// state based on messages in another. This is a style used in libraries like egui, but is adapted here for flo_scene
/// to support a wide range of possible interactions.
///
pub async fn immediate_mode_subprogram<TInput, TState, TProcessFuture>(
    input:          InputStream<TInput>, 
    context:        SceneContext, 
    update_state:   impl 'static + Send + Sync + for<'a> Fn(BoxFuture<'a, Option<Vec<TInput>>>, TState, SceneContext) -> BoxFuture<'a, Option<TState>>, 
    process_state:  impl 'static + Send + FnOnce(ImContext<TInput, TState>, SceneContext) -> TProcessFuture)
where
    TInput:         SceneMessage,
    TState:         Send,
    TProcessFuture: Send + Future<Output=TState>,
{
    // Create the ImContext
    let im_context = ImContext {
        update_state:   Arc::new(update_state),
        input_stream:   input.ready_chunks(100),
        context:        context.clone(),
    };

    // Process input in blocks
    process_state(im_context, context).await;
}
