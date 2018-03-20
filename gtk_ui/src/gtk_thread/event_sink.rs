use super::super::gtk_event::*;

use futures::prelude::*;
use futures::stream::Stream;
use futures::sink::Sink;
use futures::task;
use futures::task::Task;

use std::mem;
use std::sync::*;
use std::collections::VecDeque;

///
/// Core data for a Gtk event sink or stream
/// 
struct GtkEventCore {
    /// Events waiting to be sent to listening streams
    pending: VecDeque<GtkEvent>,

    /// Tasks waiting for an event to arrive in the pending list
    listening: Vec<Task>,

    /// Tasks waiting for the pending queue to drain
    poll_complete: Vec<Task>,

    /// Count of the number of streams that are listening for events
    active_streams: usize
}

///
/// Cloneable event sink for Gtk events
/// 
/// Gtk events are dispatched with a multiple sender/multiple receiver system
/// 
#[derive(Clone)]
pub struct GtkEventSink {
    core: Arc<Mutex<GtkEventCore>>
}

///
/// Stream that receives future events from an event sink
/// 
pub struct GtkEventStream {
    core: Arc<Mutex<GtkEventCore>>
}

impl GtkEventSink {
    ///
    /// Creates a new event sink
    /// 
    pub fn new() -> GtkEventSink {
        let core = GtkEventCore {
            pending:        VecDeque::new(),
            listening:      vec![],
            poll_complete:  vec![],
            active_streams: 0
        };

        GtkEventSink {
            core: Arc::new(Mutex::new(core))
        }
    }
    
    ///
    /// Retrieves a stream for reading from this sink
    /// 
    pub fn get_stream(&self) -> GtkEventStream {
        GtkEventStream {
            core: Arc::clone(&self.core)
        }
    }
}

impl Sink for GtkEventSink {
    type SinkItem   = GtkEvent;
    type SinkError  = ();

    fn start_send(&mut self, item: GtkEvent) -> StartSend<GtkEvent, ()> {
        // Get the core
        let mut core = self.core.lock().unwrap();

        // If there are active streams, then post the event and wake any up that are waiting
        if core.active_streams > 0 {
            // Post the event
            core.pending.push_front(item);

            // Wake the streams
            let mut listening = vec![];
            mem::swap(&mut listening, &mut core.listening);

            for task in listening {
                task.notify();
            }
        }

        // Item went to the sink
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        let mut core = self.core.lock().unwrap();

        if core.pending.len() > 0 {
            // There are tasks waiting to be consumed
            core.poll_complete.push(task::current());

            Ok(Async::NotReady)
        } else {
            // There are no tasks waiting to be consumed
            Ok(Async::Ready(()))
        }
    }
}