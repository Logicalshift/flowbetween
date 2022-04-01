///
/// Used to send the number of items available for undo/redo
///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UndoLogSize {
    pub undo_depth: usize,
    pub redo_depth: usize
}
