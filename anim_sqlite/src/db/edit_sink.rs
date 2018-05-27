use super::*;
use super::animation_core::*;

use animation::*;
use futures::*;
use futures::task;

use std::sync::*;
use std::collections::VecDeque;

///
/// The core stores the edits that are waiting to commit (and the task that should be notified when the
/// edits are complete)
///
struct EditSinkCore {
    /// Edits pending being sent to the database
    pending: VecDeque<Vec<AnimationEdit>>,

    /// Task that will be signalled when the pending queue is empty
    queue_empty_notification: Option<task::Task>
}

///
/// Sink that sends animation edits to the database
///
pub struct EditSink<TFile: FloFile+Send> {
    /// The animation core where this will send edits
    db: Arc<Desync<AnimationDbCore<TFile>>>,

    /// The sink core, which contains the edits waiting to be committed
    core: Arc<Mutex<EditSinkCore>>
}

impl<TFile: FloFile+Send+'static> EditSink<TFile> {
    ///
    /// Creates a new 
    ///
    pub fn new(db: &Arc<Desync<AnimationDbCore<TFile>>>) -> EditSink<TFile> {
        let db = Arc::clone(db);

        let core = EditSinkCore {
            pending:                    VecDeque::new(),
            queue_empty_notification:   None
        };

        EditSink {
            db:     db,
            core:   Arc::new(Mutex::new(core))
        }
    }

    ///
    /// Queues a single dequeue/commit from the pending list on the database
    ///
    fn queue_edit_dequeue(&self) {
        // Create a reference to the core that we'll use later on
        let core = Arc::clone(&self.core);

        // Queue a dequeue operation on the database
        self.db.async(move |db| {
            // Pop the next set of edits
            if let Some(edits) = core.lock().unwrap().pending.pop_front() {
                // Apply element IDs to the edits
                let edits = db.assign_element_ids(edits);

                // Add to the edit log
                db.failure = db.failure.take().or_else(|| db.insert_edits(&edits).err());

                // Perform the edits to the underlying data as well (provided the database error is clear)
                if db.failure.is_none() {
                    // Queue the edits for a single transaction
                    db.db.begin_queuing();

                    // Perform the edits
                    for edit in edits {
                        db.failure = db.failure.take().or_else(|| db.perform_edit(edit).err());
                    }

                    // Update the database and set the final error, if there was one
                    let execute_result  = db.db.execute_queue();
                    db.failure          = db.failure.take().or_else(move || execute_result.err());
                }
            }

            // Signal the task if the core is free of any further pending edits
            {
                let mut core = core.lock().unwrap();

                if core.pending.len() == 0 {
                    if let Some(notify) = core.queue_empty_notification.take() {
                        notify.notify();
                    }
                }
            }
        });
    }
}

impl<TFile: FloFile+Send+'static> Sink for EditSink<TFile> {
    type SinkItem = Vec<AnimationEdit>;
    type SinkError = ();

    fn start_send(&mut self, item: Vec<AnimationEdit>) -> StartSend<Vec<AnimationEdit>, ()> {
        // Queue this edit
        let mut sink_core = self.core.lock().unwrap();
        sink_core.pending.push_back(item);

        // Queue the performance of the edit
        self.queue_edit_dequeue();

        // Edit performed
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        let mut sink_core = self.core.lock().unwrap();

        if sink_core.pending.len() == 0 {
            // If there are no pending ends, then the edits are completed
            Ok(Async::Ready(()))
        } else {
            // If there are pending edits, then note the task and indicate that we're still processing
            sink_core.queue_empty_notification = Some(task::current());
            Ok(Async::NotReady)
        }
    }
}
