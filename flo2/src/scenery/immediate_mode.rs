// FlowBetween, a tool for creating vector animations
// Copyright (C) 2026 Andrew Hunter
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use flo_scene::*;

use futures::prelude::*;
use futures::future::{BoxFuture};

use std::sync::*;

///
/// Result from requesting
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImUpdateResult<TState> {
    /// New state to return to the processing program
    NewState(TState),

    /// Request more input and re-enter the update function
    MoreInput(TState),

    /// Updating is finished, there's no new state
    Finished,
}

///
/// Context for an immediate-mode subprogram
///
pub struct ImContext<TInput, TState> {
    /// Function that handles updating the state. Called when waiting for a new state, should await the future to retrieve the input for the current program
    update_state: Arc<dyn Send + Sync + for<'a> Fn(BoxFuture<'a, Option<Vec<TInput>>>, TState, SceneContext) -> BoxFuture<'a, ImUpdateResult<TState>>>,

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
        let context         = &self.context;
        let update_state    = &mut self.update_state;

        let mut state       = state;

        loop {
            let next_input = input_stream.next();

            state = match update_state(next_input.boxed(), state, context.clone()).await {
                ImUpdateResult::MoreInput(state)    => state,
                ImUpdateResult::NewState(state)     => { return Some(state); },
                ImUpdateResult::Finished            => { return None; },
            }
        }
    }
}

///
/// An 'immediate mode' subprogram is one that runs in two phases: it updates some state in one phase and updates that
/// state based on messages in another. This is a style used in libraries like egui, but is adapted here for flo_scene
/// to support a wide range of possible interactions.
///
/// This takes two functions. 
///
/// `update_state()` takes the previous state along with a future that receives the input for the subprogram and performs
/// any actions required to reflect any state updates (both from the last pass through the processing loop and from any
/// input received from its future)
///
/// `process_state()` runs the 'immediate mode' loop. It is responsible for creating the initial state, and then typically
/// performs a loop waiting on `ImContext::next()` to retrieve the next state. This creates the 'immediate mode' event
/// handling loop.
///
pub async fn immediate_mode_subprogram<TInput, TState, TProcessFuture>(
    input:          InputStream<TInput>, 
    context:        SceneContext, 
    update_state:   impl 'static + Send + Sync + for<'a> Fn(BoxFuture<'a, Option<Vec<TInput>>>, TState, SceneContext) -> BoxFuture<'a, ImUpdateResult<TState>>, 
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
