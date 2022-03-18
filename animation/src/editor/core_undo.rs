use super::stream_animation_core::*;
use crate::traits::*;

use futures::prelude::*;

use std::sync::*;

impl StreamAnimationCore {
    ///
    /// Performs an undo edit on this animation
    ///
    /// Undo edits are unusual in that they don't generate a 'reverse' operation (as they remove entries from the edit log), but they
    /// can change what's reported when retiring the edit. In particular a `PerformUndo` operation can become a `CompletedUndo`
    /// later on.
    ///
    pub fn undo_edit<'a>(&'a mut self, original_edit: &'a mut AnimationEdit, edit: &'a UndoEdit) -> impl 'a + Future<Output=()> {
        async move {
            use UndoEdit::*;

            match edit {
                PrepareToUndo(_name)                            => { /* Sent straight to the retired stream for synchronisation */ }
                CompletedUndo                                   => { /* original_edit is changed to this if PerformUndo is successful */ }
                FailedUndo(_reason)                             => { /* original_edit is changed to this if PerformUndo is unsuccessful */ }
                BeginAction                                     => { /* Sent straight to the retired stream for organization */ }
                FinishAction                                    => { /* Sent straight to the retired stream for organization */ },
                PerformUndo { original_actions, undo_actions }  => {
                    // The original edit is updated according to whether or not the undo succeeds or fails
                    match self.perform_undo(Arc::clone(original_actions), Arc::clone(undo_actions)).await {
                        Ok(())      => *original_edit = AnimationEdit::Undo(CompletedUndo),
                        Err(fail)   => *original_edit = AnimationEdit::Undo(FailedUndo(fail))
                    }
                }
            }
        }
    }

    ///
    /// Carries out an undo operation if possible
    ///
    pub fn perform_undo<'a>(&'a mut self, original_actions: Arc<Vec<AnimationEdit>>, undo_actions: Arc<Vec<AnimationEdit>>) -> impl 'a + Future<Output=Result<(), UndoFailureReason>> {
        async move {
            Err(UndoFailureReason::NotSupported)
        }
    }
}
