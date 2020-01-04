use super::command::*;

///
/// Possible types of output from a FloCommand
///
#[derive(Clone)]
pub enum FloCommandOutput {
    /// A particular command has started running
    BeginCommand(FloCommand),

    /// Display a message to the user
    Message(String),

    /// Display an error message to the user
    Error(String),

    /// A command has finished running
    FinishCommand(FloCommand)
}
