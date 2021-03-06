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

    /// Flag indicating the command should only be generated 'internally'
    pub is_system: bool,

    /// A name that can be used to describe this command to the user
    pub name: String,

    /// The names of the parameters required to run this command
    pub required_parameters: Option<Vec<String>>
}

impl Command {
    ///
    /// Creates a new command with only an ID
    ///
    pub fn with_id<ToString: Into<String>>(identifier: ToString) -> Command {
        Command { 
            identifier:             identifier.into(),
            is_system:              false,
            name:                   String::new(),
            required_parameters:    None
        }
    }

    ///
    /// Creates a new system command with the specified ID
    ///
    /// 'System' commands cannot be directly triggered by the user and are hidden from
    /// command palettes, and which occupy a separate namespace, so they won't be confused
    /// with 'standard' commands when they're used this way.
    ///
    /// These are intended for the case where a command has 'unsafe' parameters such
    /// as a local filename and we don't want a user to be able to specify that directly.
    /// 
    /// System commands cannot be generated via a remote request or via a command palette.
    ///
    pub fn system_with_id<ToString: Into<String>>(identifier: ToString) -> Command {
        Command { 
            identifier:             identifier.into(),
            is_system:              true,
            name:                   String::new(),
            required_parameters:    None
        }
    }

    ///
    /// Returns a command with an added description
    ///
    pub fn named<ToString: Into<String>>(self, name: ToString) -> Command {
        let mut cmd = self;
        cmd.name = name.into();
        cmd
    }

    ///
    /// Returns a command with a set of parameter descriptions
    ///
    pub fn parameters<ParamIter: IntoIterator>(self, parameters: ParamIter) -> Command 
    where ParamIter::Item : Into<String> {
        let mut cmd = self;

        let parameters = parameters.into_iter().map(|item| item.into()).collect::<Vec<_>>();
        let parameters = if parameters.len() > 0 { Some(parameters) } else { None };

        cmd.required_parameters = parameters;

        cmd
    }
}

impl PartialEq for Command {
    ///
    /// Two commands are equal if they have the same identifier
    ///
    fn eq(&self, other: &Command) -> bool {
        self.identifier.eq(&other.identifier) && self.is_system.eq(&other.is_system)
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
        self.is_system.hash(state);
    }
}
