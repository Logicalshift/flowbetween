use super::undo_log::*;
use super::undo_log_size::*;
use super::edit_log_reader::*;
use super::undoable_animation_core::*;
use crate::traits::*;

use futures::prelude::*;
use futures::stream::{BoxStream};

use ::desync::*;
use flo_stream::*;

use uuid::*;

use std::sync::*;
use std::time::{Duration};
use std::ops::{Range};

///
/// Adds undo support to another animation type
///
pub struct UndoableAnimation<Anim: 'static+Unpin+EditableAnimation> {
    /// The animation that this will add undo support to
    animation:      Arc<Desync<Anim>>,

    /// The actions to undo or redo in the animation
    core:           Arc<Desync<UndoableAnimationCore>>,
}

impl<Anim: 'static+Unpin+EditableAnimation> UndoableAnimation<Anim> {
    ///
    /// Adds undo support to an existing animation
    ///
    pub fn new(animation: Anim) -> UndoableAnimation<Anim> {
        // Box up the animation and create the edit stream
        let animation           = Arc::new(Desync::new(animation));
        let undo_log            = UndoLog::new();
        let log_size_publisher  = ExpiringPublisher::new(1);

        let core                = UndoableAnimationCore {
            undo_log,
            log_size_publisher
        };
        let core                = Arc::new(Desync::new(core));

        // Set up communication with the animation and with the undo log
        Self::pipe_retired_edits_to_undo_log(&animation, &core);

        UndoableAnimation {
            animation,
            core,
        }
    }

    ///
    /// When the underlying animation retires its edits, send them to the undo log
    ///
    fn pipe_retired_edits_to_undo_log(animation: &Arc<Desync<Anim>>, core: &Arc<Desync<UndoableAnimationCore>>) {
        let retired_edits = animation.sync(|anim| anim.retired_edits());

        pipe_in(Arc::clone(core), retired_edits, move |core, retired_edits| {
            async move {
                core.undo_log.retire(retired_edits);
                core.update_undo_log_size().await;
            }.boxed()
        });
    }

    ///
    /// Reads the next element from a retired edits stream and returns either `None` to indicate it's not an undo result,
    /// `Some(Ok(()))` to indicate success or `Some(Err(_)))` to indicate failure
    ///
    fn read_undo_completion<'a>(retired_edits: &'a mut (impl Stream<Item=RetiredEdit> + Unpin)) -> impl 'a+Future<Output=Option<Result<(), UndoFailureReason>>> {
        async move {
            // Read the next action
            let action      = retired_edits.next().await;
            let action      = if let Some(action) = action { action } else { return Some(Err(UndoFailureReason::BadEditingSequence)); };

            // If the next action is an undo completion or failure action, then indicate success or failure
            let committed   = action.committed_edits();
            if committed.len() == 1 {
                match committed[0] {
                    AnimationEdit::Undo(UndoEdit::CompletedUndo(_)) => Some(Ok(())),
                    AnimationEdit::Undo(UndoEdit::FailedUndo(err))  => Some(Err(err)),
                    _                                               => None,
                }
            } else {
                // Too long to be the completion action
                None
            }
        }
    }

    ///
    /// Undoes the last action performed on this animation
    ///
    pub fn undo<'a>(&'a self) -> impl 'a + Future<Output=Result<(), UndoFailureReason>> {
        async move {
            // Scheduling on the animation desync will prevent any further edits from occurring while we're performing the undo
            let core = self.core.clone();

            self.animation.future_desync(move |animation| {
                async move {
                    // We'll monitor the retired edits from the animation
                    let mut retired_edits   = animation.retired_edits();

                    // Use 'prepare to undo' to ensure that all the edits have retired
                    let id                  = Uuid::new_v4().to_simple().to_string();
                    let prepare_undo        = Arc::new(vec![AnimationEdit::Undo(UndoEdit::PrepareToUndo(id))]);
                    animation.edit().publish(Arc::clone(&prepare_undo)).await;

                    // Process edits from retired_edits until the 'prepare' event is relayed back to us
                    while let Some(edit) = retired_edits.next().await {
                        if edit.committed_edits() == prepare_undo {
                            break;
                        }
                    }

                    // Fetch the undo action that we're about to perform
                    let undo_edit = core.future_sync(|core| async move {
                        core.undo_log.start_undoing();
                        core.undo_log.undo()
                    }.boxed()).await.unwrap();

                    let undo_edit = if let Some(undo_edit) = undo_edit { 
                        undo_edit 
                    } else { 
                        core.future_sync(|core| async move { 
                            core.undo_log.finish_undoing(); 
                            core.update_undo_log_size().await;
                        }.boxed()).await.unwrap();
                        return Err(UndoFailureReason::NothingToUndo); 
                    };

                    // Carry out the undo action on the animation
                    animation.edit().publish(Arc::new(vec![AnimationEdit::Undo(undo_edit)])).await;

                    // A failure will produce a single retired edit, and a success will produce two, so read up to two edits
                    let undo_result = Self::read_undo_completion(&mut retired_edits).await;
                    let undo_result = if let Some(undo_result) = undo_result { Some(undo_result) } else { Self::read_undo_completion(&mut retired_edits).await };

                    // The undo is complete at this point
                    // Note: we're relying on the edit to have been queued in sequence here so that the 'finish_undoing' happens after the
                    // undo log pipe has received this edit
                    core.future_sync(|core| async move { 
                        core.undo_log.finish_undoing(); 
                        core.update_undo_log_size().await;
                    }.boxed()).await.unwrap();

                    match undo_result {
                        Some(Ok(()))        => Ok(()),
                        Some(Err(failure))  => Err(failure),
                        None                => Err(UndoFailureReason::BadEditingSequence),
                    }
                }.boxed()
            }).await.unwrap()
        }
    }

    ///
    /// Redoes the last action that was undone by undo
    ///
    pub fn redo<'a>(&'a self) -> impl 'a + Future<Output=Result<(), UndoFailureReason>> {
        async move {
            // Scheduling on the animation desync will prevent any further edits from occurring while we're performing the undo
            let core = self.core.clone();

            self.animation.future_desync(move |animation| {
                async move {
                    // We'll monitor the retired edits from the animation
                    let mut retired_edits   = animation.retired_edits();

                    // Use 'prepare to undo' to ensure that all the edits have retired
                    let id                  = Uuid::new_v4().to_simple().to_string();
                    let prepare_undo        = Arc::new(vec![AnimationEdit::Undo(UndoEdit::PrepareToUndo(id))]);
                    animation.edit().publish(Arc::clone(&prepare_undo)).await;

                    // Process edits from retired_edits until the 'prepare' event is relayed back to us 
                    // (as we're blocking the animation, no other actions will interfere with this redo action)
                    while let Some(edit) = retired_edits.next().await {
                        if edit.committed_edits() == prepare_undo {
                            break;
                        }
                    }

                    // Fetch the redo action that we're about to perform
                    let redo_edit = core.future_sync(|core| async move {
                        core.undo_log.start_undoing();
                        core.undo_log.redo()
                    }.boxed()).await.unwrap();

                    let redo_edit = if let Some(redo_edit) = redo_edit { 
                        redo_edit 
                    } else { 
                        core.future_sync(|core| async move { 
                            core.undo_log.finish_undoing(); 
                            core.update_undo_log_size().await;
                        }.boxed()).await.unwrap();
                        return Err(UndoFailureReason::NothingToRedo); 
                    };

                    // Carry out the redo action on the animation
                    animation.edit().publish(redo_edit).await;

                    // Wait for the redo edit to retire
                    // Note: we're relying on the edit to have been queued in sequence here so that the 'finish_undoing' happens after the
                    // undo log pipe has received this edit
                    retired_edits.next().await;

                    // The redo is complete at this point
                    core.future_sync(|core| async move { 
                        core.undo_log.finish_undoing(); 
                        core.update_undo_log_size().await;
                    }.boxed()).await.unwrap();

                    Ok(())
                }.boxed()
            }).await.unwrap()
        }
    }

    ///
    /// Retrieves a stream that tracks the size of the undo log (this is an expiring stream, so backpressure will cause
    /// updates to be discarded: ie, reads will always return the lastest value)
    ///
    pub fn follow_undo_log_size_changes(&self) -> impl Send + Sync + Stream<Item=UndoLogSize> {
        let log_size_stream = self.core.sync(|core| core.log_size_publisher.subscribe());

        self.core.future_desync(|core| async move { core.update_undo_log_size().await; }.boxed()).detach();

        log_size_stream
    }
}

impl<Anim: 'static+Unpin+EditableAnimation> Animation for UndoableAnimation<Anim> {
    ///
    /// Retrieves the frame size of this animation
    ///
    fn size(&self) -> (f64, f64) { 
        self.animation.sync(|anim| anim.size()) 
    }

    ///
    /// Retrieves the length of this animation
    ///
    fn duration(&self) -> Duration { 
        self.animation.sync(|anim| anim.duration()) 
    }

    ///
    /// Retrieves the duration of a single frame
    ///
    fn frame_length(&self) -> Duration { 
        self.animation.sync(|anim| anim.frame_length()) 
    }

    ///
    /// Retrieves the IDs of the layers in this object
    ///
    fn get_layer_ids(&self) -> Vec<u64> { 
        self.animation.sync(|anim| anim.get_layer_ids()) 
    }

    ///
    /// Retrieves the layer with the specified ID from this animation
    ///
    fn get_layer_with_id(&self, layer_id: u64) -> Option<Arc<dyn Layer>> { 
        self.animation.sync(|anim| anim.get_layer_with_id(layer_id)) 
    }

    ///
    /// Retrieves the total number of items that have been performed on this animation
    ///
    fn get_num_edits(&self) -> usize { 
        self.animation.sync(|anim| anim.get_num_edits()) 
    }

    ///
    /// Reads from the edit log for this animation
    ///
    fn read_edit_log<'a>(&'a self, range: Range<usize>) -> BoxStream<'a, AnimationEdit> {
        read_desync_edit_log(Arc::clone(&self.animation), range).boxed()
    }
}

impl<Anim: 'static+Unpin+EditableAnimation> EditableAnimation for UndoableAnimation<Anim> {
    ///
    /// Assigns a new unique ID for creating a new motion
    ///
    /// This ID will not have been used so far and will not be used again, and can be used as the ID for the MotionElement vector element.
    ///
    fn assign_element_id(&self) -> ElementId {
        self.animation.sync(|anim| anim.assign_element_id())
    }

    ///
    /// Retrieves a sink that can be used to send edits for this animation
    ///
    /// Edits are supplied as groups (stored in a vec) so that it's possible to ensure that
    /// a set of related edits are performed atomically
    ///
    fn edit(&self) -> Publisher<Arc<Vec<AnimationEdit>>> {
        self.animation.sync(|anim| anim.edit())
    }

    ///
    /// Sends a set of edits straight to this animation
    /// 
    /// (Note that these are not always published to the publisher)
    ///
    fn perform_edits(&self, edits: Vec<AnimationEdit>) {
        self.animation.sync(|anim| anim.perform_edits(edits));
    }

    ///
    /// Returns a stream of edits as they are being retired (ie, the edits that are now visible on the animation)
    ///
    fn retired_edits(&self) -> BoxStream<'static, RetiredEdit> {
        self.animation.sync(|anim| anim.retired_edits())
    }

    ///
    /// Flushes any caches this might have (forces reload from data storage)
    ///
    fn flush_caches(&self) {
        self.animation.sync(|anim| anim.flush_caches())
    }
}
