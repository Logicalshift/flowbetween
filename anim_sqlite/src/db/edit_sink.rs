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
    pending: VecDeque<AnimationEdit>,

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

impl<TFile: FloFile+Send> EditSink<TFile> {

}

impl<TFile: FloFile+Send> Sink for EditSink<TFile> {
    type SinkItem = Vec<AnimationEdit>;
    type SinkError = ();

    fn start_send(&mut self, item: Vec<AnimationEdit>) -> StartSend<Vec<AnimationEdit>, ()> {
        // Edit performed
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        // The in-memory sink performs all edits immediately, so is ever-ready
        Ok(Async::Ready(()))
    }
}
