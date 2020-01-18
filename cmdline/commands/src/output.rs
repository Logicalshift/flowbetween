use super::state::*;
use super::error::*;
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

    /// Starts writing output to a particular file
    BeginOutput(String),

    /// Generates output for saving
    Output(String),

    /// Display an error message to the user
    Error(String),

    /// Retrieved the current state of the command line tool
    State(CommandState),

    /// A command has finished running
    FinishCommand(FloCommand),

    /// We're starting a task
    StartTask(String),

    /// We've made x/y progress on a command
    TaskProgress(f64, f64),

    /// The last task started with StartTask has finished
    FinishTask,

    /// A command generated an error (this is generally the last item in the stream)
    Failure(CommandError)
}
