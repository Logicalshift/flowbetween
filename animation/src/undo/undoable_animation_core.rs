use super::undo_log::*;
use super::undo_log_size::*;

use flo_stream::*;

///
/// The data structures used to store the state of an undoable animation
///
pub struct UndoableAnimationCore {
    /// The undo log stores the list of undo and redo actions
    pub (super) undo_log: UndoLog,

    /// Publisher that can be used to track changes to the number of entries in the undo log 
    pub (super) log_size_publisher: ExpiringPublisher<UndoLogSize>,
}


impl UndoableAnimationCore {
    ///
    /// Sends the size of the undo log to anything that's subscribed to the log size publisher
    ///
    pub async fn update_undo_log_size(&mut self) {
        let undo_depth  = self.undo_log.undo_depth();
        let redo_depth  = self.undo_log.redo_depth();

        let log_size    = UndoLogSize {
            undo_depth,
            redo_depth
        };

        self.log_size_publisher.publish(log_size).await;
    }
}