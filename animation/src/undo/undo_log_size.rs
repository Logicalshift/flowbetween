///
/// Used to send the number of items available for undo/redo
///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UndoLogSize {
    undo_depth: usize,
    redo_depth: usize
}
