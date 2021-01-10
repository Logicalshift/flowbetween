use std::hash::{Hash, Hasher};

///
/// A command is used to send a message to a UI component without specifically knowing how to operate it.
///
/// Commands are usually bound to a single controller, but in the event that they're bound to multiple places,
/// the command is sent to all of the places it's bound. A command may optional have a set of required parameters
/// which are specified by giving them names.
///
/// Commands can be used for numerous functions:
///     * Keyboard shortcuts can invoke commands
///     * Menu items invoke commands
///     * Control actions can invoke commands
///     * Command palettes can provide a way to find all available commands in the application and a shortcut to them
///     * Tests can use commands to test the UI 'headless' (FlowBetween's 'split' UI design makes true headless operation of the UI practical)
///
/// Two commands are considered the same if they have the same ID: the other parts (name, required parameters) are 
/// only used to describe the command to the user.
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    /// The identifier used to call this command from the code
    pub identifier: String,

    /// A name that can be used to describe this command to the user
    pub name: String,

    /// The names of the parameters required to run this command
    pub required_parameters: Option<Vec<String>>
}

impl Command {
    ///
    /// Creates a new command with only an ID
    ///
    pub fn with_id(identifier: String) -> Command {
        Command { 
            identifier:             identifier,
            name:                   String::new(),
            required_parameters:    None
        }
    }

    ///
    /// Creates a command with a user description
    ///
    pub fn with_name(identifier: String, name: String) -> Command {
        Command {
            identifier:             identifier,
            name:                   name,
            required_parameters:    None
        }
    }

    ///
    /// Creates a command with a user description and a specified number of required parameters
    ///
    pub fn with_parameters<ParamIter: IntoIterator<Item=String>>(identifier: String, name: String, parameters: ParamIter) -> Command {
        let parameters = parameters.into_iter().collect::<Vec<_>>();
        let parameters = if parameters.len() == 0 { None } else { Some(parameters) };

        Command {
            identifier:             identifier,
            name:                   name,
            required_parameters:    parameters
        }
    }
}

impl PartialEq for Command {
    ///
    /// Two commands are equal if they have the same identifier
    ///
    fn eq(&self, other: &Command) -> bool {
        self.identifier.eq(&other.identifier)
    }
}

impl Eq for Command {
}

impl Hash for Command {
    ///
    /// Commands are hashed by their identifier only
    ///
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
    }
}
