use super::undo_step::*;
use crate::traits::*;

///
/// A log of undo elements
///
pub struct UndoLog {
    /// The list of undo steps, with the latest at the end
    undo: Vec<UndoStep>,

    /// Steps that have been undone and which can be re-done
    redo: Vec<UndoStep>,
}

impl UndoLog {
    ///
    /// Creates a new empty undo log
    ///
    pub fn new() -> UndoLog {
        UndoLog {
            undo: vec![],
            redo: vec![],
        }
    }

    ///
    /// Retires an edit to this undo log
    ///
    pub fn retire(&mut self, edit: RetiredEdit) {
        // Create the initial undo step if needed
        if self.undo.is_empty() {
            self.undo.push(UndoStep::new());
        }

        // Any redo actions are destroyed when a new action is created
        self.redo.drain(..);

        // Determine if the edit finishes an action group
        let finishes_action_group = edit.committed_edits().iter().any(|edit| match edit {
            AnimationEdit::Undo(UndoEdit::FinishAction) => true,
            _                                           => false,
        });

        // Add the edit to the current undo step
        self.undo.last_mut().unwrap().push_edit(edit);

        // Start a new action group if this edit finished one
        if finishes_action_group {
            self.undo.push(UndoStep::new());
        }
    }

    ///
    /// Pops the action on top of the 
    ///
    pub fn undo(&mut self) -> Option<UndoEdit> {
        // Pop up to two actions (in case the first one is empty)
        let most_recent_action = self.undo.pop()?;
        let most_recent_action = if most_recent_action.is_empty() { self.undo.pop()? } else { most_recent_action };

        let undo_edit = most_recent_action.undo_edit();

        // Add as a redo action
        self.redo.push(most_recent_action);

        // Add a new action group to the undo list so if there are any future actions, they won't extend an existing one
        self.undo.push(UndoStep::new());

        Some(undo_edit)
    }
}
