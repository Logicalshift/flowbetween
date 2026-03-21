use flo_scene::*;

use futures::prelude::*;
use futures::future::{BoxFuture};

///
/// An 'immediate mode' subprogram is one that runs in two phases: it updates some state in one phase and updates that
/// state based on messages in another. This is a style used in libraries like egui, but is adapted here for flo_scene
/// to support a wide range of possible interactions.
///
pub async fn immediate_mode_subprogram<TInput, TState, TStateFuture>(
    input:          InputStream<TInput>, 
    context:        SceneContext, 
    update_state:   impl Send + FnMut(Vec<TInput>, TState, SceneContext) -> TStateFuture, 
    process_state:  impl Send + for<'a> FnMut(&'a mut TState, SceneContext) -> BoxFuture<'a, ()>)
where
    TInput:         SceneMessage,
    TState:         Send + Default,
    TStateFuture:   Send + Future<Output=TState>,
{
    // Create the initial state
    let mut state = TState::default();

    // Process input in blocks
    let mut input           = input.ready_chunks(100);
    let mut update_state    = update_state;
    let mut process_state   = process_state;

    while let Some(input) = input.next().await {
        state = update_state(input, state, context.clone()).await;
        process_state(&mut state, context.clone()).await;
    }
}
