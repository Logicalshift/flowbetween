use crate::traits::*;

use std::sync::*;

///
/// An undo step represents the commands that will be processed by an undo or redo operation
///
#[derive(Clone)]
pub struct UndoStep {
    ///
    /// The edits that make up this step
    ///
    edits: Vec<RetiredEdit>,
}

impl UndoStep {
    ///
    /// Creates a new undo step
    ///
    pub fn new() -> UndoStep {
        UndoStep {
            edits: vec![]
        }
    }

    ///
    /// True if this step is empty (eg, due to being just after a finished action)
    ///
    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }

    ///
    /// Adds a new edit to the list known about by this step
    ///
    pub fn push_edit(&mut self, edit: RetiredEdit) {
        self.edits.push(edit);
    }

    ///
    /// Creates the undo edit for this step
    ///
    pub fn undo_edit(&self) -> UndoEdit {
        let original_actions    = self.edits.iter().flat_map(|edit| edit.committed_edits().iter().cloned().collect::<Vec<_>>()).collect();
        let undo_actions        = self.edits.iter().flat_map(|edit| edit.reverse_edits().iter().cloned().collect::<Vec<_>>()).collect();
        let original_actions    = Arc::new(original_actions);
        let undo_actions        = Arc::new(undo_actions);

        UndoEdit::PerformUndo { original_actions, undo_actions }
    }
}
