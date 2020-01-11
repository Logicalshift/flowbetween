use super::storage_descriptor::*;

use flo_animation::*;
use flo_anim_sqlite::*;

use std::sync::*;

///
/// Represents the state of a command stream
///
#[derive(Clone)]
pub struct CommandState(Arc<StateValue>);

///
/// How an animation is stored within the command state
///
#[derive(Clone)]
struct AnimationState(StorageDescriptor, Arc<dyn Animation>);

///
/// The internal value of a command state
///
struct StateValue { 
    /// The animation that this will read from
    input_animation: AnimationState,

    /// The animation that this will write to
    output_animation: AnimationState
}

impl CommandState {
    ///
    /// Creates a new command state with the default settings
    ///
    pub fn new() -> CommandState {
        // Create the input and output animation in memory
        let input_animation     = SqliteAnimation::new_in_memory();
        let output_animation    = SqliteAnimation::new_in_memory();
        let input_animation     = AnimationState(StorageDescriptor::InMemory, Arc::new(input_animation));
        let output_animation    = AnimationState(StorageDescriptor::InMemory, Arc::new(output_animation));

        // Generate the initial command state
        CommandState(Arc::new(StateValue {
            input_animation:    input_animation,
            output_animation:   output_animation
        }))
    }
}
