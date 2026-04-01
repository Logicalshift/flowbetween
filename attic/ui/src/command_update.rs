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

impl CommandUpdate {
    ///
    /// Returns the command that is affected by this update
    ///
    pub fn command(&self) -> Option<&Command> {
        use self::CommandUpdate::*;

        match self {
            Add(cmd)    => Some(cmd),
            Remove(cmd) => Some(cmd)
        }
    }

    ///
    /// Returns true if the command affected by this update is a system command
    ///
    pub fn is_system_command(&self) -> bool {
        self.command()
            .map(|cmd| cmd.is_system)
            .unwrap_or(false)
    }
}
