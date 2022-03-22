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
    pub fn retire_edit(&mut self, edit: RetiredEdit) {
        // Create the initial undo step if needed
        if self.undo.is_empty() {
            self.undo.push(UndoStep::new());
        }

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
}
