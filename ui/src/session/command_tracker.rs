use super::command_update::*;
use crate::control::*;

use std::collections::{HashSet, HashMap};

///
/// Tracks which commands are published by controllers
///
pub struct CommandTracker {
    /// The commands in the controller that this is tracking
    commands: HashSet<Command>,
}

impl CommandTracker {
    ///
    /// Creates a new command tracker
    ///
    pub fn new() -> CommandTracker {
        CommandTracker {
            commands:           HashSet::new()
        }
    }

    ///
    /// Fills a hashset with the commands in the specified control
    ///
    fn fill_command_hashset(control: &Control, commands: &mut HashSet<Command>) {
        // Add every command that can be triggered on this control
        for attr in control.attributes() {
            match attr {
                ControlAttribute::Action(ActionTrigger::Command(cmd), _)    => { if !commands.contains(cmd) { commands.insert(cmd.clone()); } }
                _                                                           => { }
            }
        }

        // Add the commands for the subcomponents of this control
        if let Some(subcomponents) = control.subcomponents() {
            for subcomponent in subcomponents.iter() {
                Self::fill_command_hashset(subcomponent, commands);
            }
        }
    }

    ///
    /// Updates the list of commands in this object and returns the changes
    ///
    pub fn update_from_ui(&mut self, ui: &Control) -> Vec<CommandUpdate> {
        // Get the original set of commands and generate a new set
        let original_commands   = &self.commands;
        let mut new_commands    = HashSet::new();

        // Fill out the set of new commands
        Self::fill_command_hashset(ui, &mut new_commands);

        // Inspect the two sets of commands for differences
        let mut updates = vec![];

        for original in original_commands.iter() {
            if !new_commands.contains(original) {
                updates.push(CommandUpdate::Remove(original.clone()));
            }
        }

        for new in new_commands.iter() {
            if !original_commands.contains(new) {
                updates.push(CommandUpdate::Add(new.clone()));
            }
        }

        // Update the commands in this control
        self.commands           = new_commands;

        updates
    }
}
