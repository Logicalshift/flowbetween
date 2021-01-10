use crate::control::*;

///
/// Describes an update to the set of commands that can be evaluated on a UI session
///
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum CommandUpdate {
    /// A new command is available to be evaluated
    Add(Command),

    /// A command that was previously available (via Add) has been removed
    Remove(Command)
}
