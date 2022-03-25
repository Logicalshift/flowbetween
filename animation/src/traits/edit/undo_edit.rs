use super::animation_edit::*;
use crate::storage::*;

use std::sync::*;

///
/// Reasons why a PerformUndo might fail
///
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum UndoFailureReason {
    /// The undo operation was not supported by the editor
    NotSupported,

    /// There were no actions to undo
    NothingToUndo,

    /// There were no actions to redo
    NothingToRedo,

    /// There was an unexpected problem accessing the backing store
    StorageError,

    /// There are not enough edits to cover the original actions
    EditLogTooShort,

    /// The original actions could not be read from the edit log for comparison
    CannotReadOriginalActions,

    /// The actions being undone do not match the actions on top of the edit log
    OriginalActionsDoNotMatch,

    /// The animation failed to report the undo success/failure properly
    BadEditingSequence,
}

impl From<StorageError> for UndoFailureReason {
    fn from(error: StorageError) -> UndoFailureReason {
        use StorageError::*;

        match error {
            General                     |
            FailedToInitialise          |
            CannotContinueAfterError    => { UndoFailureReason::StorageError }
        }
    }
}

///
/// Undo edits do not affect the animation but instead are used to annotate the edit log to mark where 
/// undo actions occur, and to rewrite the edit log after an undo action has occurred.
///
/// Note that only BeginAction and FinishAction are serialized to the edit log: other undo actions are
/// always left out.
///
#[derive(Clone, PartialEq, Debug)]
pub enum UndoEdit {
    /// Provides a synchronisation point so that an undo action can be performed once all pending edits have been retired
    PrepareToUndo(String),

    /// A 'PerformUndo' is retired as a 'CompletedUndo' if successful. The parameter are the `undo_actions` performed to complete this operation.
    /// (Ie, when a `PerformUndo` edit is committed, the retired stream will have either a 'completed' or 'failed' edit to replace it)
    CompletedUndo(Arc<Vec<AnimationEdit>>),

    /// A 'PerformUndo' is retired as a 'FailedUndo' if unsuccessful, along with a reason why the operation couldn't be performed
    /// (Ie, when a `PerformUndo` edit is committed, the retired stream will have either a 'completed' or 'failed' edit to replace it)
    FailedUndo(UndoFailureReason),

    /// Indicates that the subsequent edit operations all form part of a single action
    BeginAction,

    /// Finishes an action started by BeginAction()
    FinishAction,

    /// Performs a set of undo actions, removing the original actions from the log (this is never serialized to the log)
    PerformUndo { original_actions: Arc<Vec<AnimationEdit>>, undo_actions: Arc<Vec<AnimationEdit>> }
}
