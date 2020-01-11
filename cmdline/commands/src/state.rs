use std::sync::*;

///
/// Represents the state of a command stream
///
#[derive(Clone)]
pub struct CommandState(Arc<StateValue>);

///
/// The internal value of a command state
///
struct StateValue { 
}

impl CommandState {
    ///
    /// Creates a new command state with the default settings
    ///
    pub fn new() -> CommandState {
        CommandState(Arc::new(StateValue {

        }))
    }
}
