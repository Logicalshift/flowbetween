use super::undo_log::*;

///
/// The data structures used to store the state of an undoable animation
///
pub struct UndoableAnimationCore {
    pub (super) undo_log: UndoLog
}