use crate::traits::*;

///
/// An undo step represents the commands that will be processed by an undo or redo operation
///
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
    /// Adds a new edit to the list known about by this step
    ///
    pub fn push_edit(&mut self, edit: RetiredEdit) {
        self.edits.push(edit);
    }
}
