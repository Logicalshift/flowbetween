use super::undo_step::*;

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
}