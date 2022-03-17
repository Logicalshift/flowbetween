use super::animation_edit::*;

use std::sync::*;

///
/// Undo edits do not affect the animation but instead are used to annotate the edit log to mark where 
/// undo actions occur, and to rewrite the edit log after an undo action has occurred.
///
#[derive(Clone, PartialEq, Debug)]
pub enum UndoEdit {
    /// Provides a synchronisation point so that an undo action can be performed once all pending edits have been retired
    PrepareToUndo(String),

    /// Indicates that the subsequent edit operations all form part of a single action
    BeginAction(String),

    /// Finishes an action started by BeginAction()
    FinishAction(String),

    /// Performs a set of undo actions, removing the original actions from the log
    PerformUndo { original_actions: Arc<Vec<AnimationEdit>>, undo_actions: Arc<Vec<AnimationEdit>> }
}
