use super::super::gtk_thread::*;
use super::super::gtk_action::*;

use futures::*;

use std::sync::*;

///
/// Sink used to send GTK actions to a thread
///
pub struct ActionSink {
    /// The thread that this sink will send its actions to
    thread: Arc<GtkThread>
}

impl ActionSink {
    ///
    /// Creates a new action sink that will send events to the specified thread
    ///
    pub fn new(thread: Arc<GtkThread>) -> ActionSink {
        ActionSink {
            thread: thread
        }
    }
}

impl Sink for ActionSink {
    type SinkItem   = Vec<GtkAction>;
    type SinkError  = ();

    fn start_send(&mut self, item: Vec<GtkAction>) -> StartSend<Vec<GtkAction>, ()> {
        // Items can always be sent directly to the thread
        self.thread.perform_actions(item);

        // Item went to the sink
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        // At the moment, we don't know when our actions have been processed, so we just always indicate readiness
        // TODO: add a way for perform_actions to indicate when its actions have been completed
        Ok(Async::Ready(()))
    }
}
