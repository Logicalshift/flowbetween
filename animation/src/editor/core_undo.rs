use super::stream_animation_core::*;
use crate::traits::*;

use flo_stream::*;
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
                CompletedUndo(_edits)                           => { /* original_edit is changed to this if PerformUndo is successful */ }
                FailedUndo(_reason)                             => { /* original_edit is changed to this if PerformUndo is unsuccessful */ }
                BeginAction                                     => { /* Sent straight to the retired stream for organization */ }
                FinishAction                                    => { /* Sent straight to the retired stream for organization */ },
                PerformUndo { original_actions, undo_actions }  => {
                    // The original edit is updated according to whether or not the undo succeeds or fails
                    match self.perform_undo(Arc::clone(original_actions), Arc::clone(undo_actions)).await {
                        Ok(())      => *original_edit = AnimationEdit::Undo(CompletedUndo(Arc::clone(undo_actions))),
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
            // Check that the original actions match what's in the edit log
            let num_original_actions    = original_actions.len();
            let edit_log_length         = self.storage_connection.read_edit_log_length().await.ok_or(UndoFailureReason::StorageError)?;

            if edit_log_length < num_original_actions { 
                return Err(UndoFailureReason::EditLogTooShort);
            }

            let edit_log_start          = edit_log_length - num_original_actions;
            let expected_actions        = self.storage_connection.read_edit_log(edit_log_start..edit_log_length).await.ok_or(UndoFailureReason::CannotReadOriginalActions)?;

            if &expected_actions != &*original_actions {
                return Err(UndoFailureReason::OriginalActionsDoNotMatch);
            }

            // Perform the undo actions, without adding them to the edit log
            let undo_actions        = self.assign_ids_to_edits(&*undo_actions).await;
            let retired_edits       = self.perform_edits(undo_actions).await;

            // Send the retired edits to anything that's listening (as if they're normal edits)
            for retired_sender in self.retired_edit_senders.iter_mut() {
                retired_sender.publish(retired_edits.clone()).await;
            }

            // Remove the actions that were undone from the storage log
            self.storage_connection.delete_recent_edits(num_original_actions).await?;

            // Undo is complete
            Ok(())
        }
    }
}
