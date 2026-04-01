use super::actions::*;
use super::command::*;
use crate::controller::*;

use std::sync::*;

///
/// Represents a binding of a command to a controller
///
#[derive(Clone)]
pub struct CommandBinding {
    /// The controller that this binding is for
    pub controller: Weak<dyn Controller>,

    /// The command this binding is for
    pub command: Command,

    /// The path to the controller
    pub path: Vec<String>,

    /// The name of the action that the command triggers in the controller
    pub action: ActionEvent
}

impl PartialEq for CommandBinding {
    fn eq(&self, other: &CommandBinding) -> bool {
        self.action.eq(&other.action) && self.path.eq(&other.path)
    }
}
