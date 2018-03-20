use super::super::gtk_event::*;

use futures::prelude::*;
use futures::stream::Stream;
use futures::sink::Sink;
use futures::task;
use futures::task::Task;

use std::sync::*;
use std::collections::VecDeque;

///
/// Core data for a Gtk event sink or stream
/// 
struct GtkEventSinkCore {
    /// True if the sink has been dropped
    dropped: bool,

    /// Streams that are attached to this event sink
    streams: Vec<Arc<Mutex<GtkEventStreamCore>>>,

    /// If something is waiting for all of the events to drain, this is the task to notify
    poll_complete: Option<Task>
}

///
/// Core data for a Gtk event stream
/// 
struct GtkEventStreamCore {
    /// Set to true if this stream has been dropped and is not in use any more
    dropped: bool,

    /// The events that have been sent to this stream and are not yet consumed
    pending: VecDeque<GtkEvent>,

    /// If the stream is waiting for an event, this is the task to notify
    poll_event: Option<Task>,
}

///
/// Cloneable event sink for Gtk events
/// 
/// Gtk events are dispatched with a multiple sender/multiple receiver system
/// 
#[derive(Clone)]
pub struct GtkEventSink {
    /// Core of this sink
    core: Arc<Mutex<GtkEventSinkCore>>
}

///
/// Stream that receives future events from an event sink
/// 
pub struct GtkEventStream {
    /// Core of this stream
    core: Arc<Mutex<GtkEventStreamCore>>,

    /// Core of the sink this stream is attached to
    sink_core: Arc<Mutex<GtkEventSinkCore>>
}

impl GtkEventSink {
    ///
    /// Creates a new event sink
    /// 
    pub fn new() -> GtkEventSink {
        let core = GtkEventSinkCore {
            dropped:        false,
            streams:        vec![],
            poll_complete:  None
        };

        GtkEventSink {
            core: Arc::new(Mutex::new(core))
        }
    }
    
    ///
    /// Retrieves a stream for reading from this sink
    /// 
    pub fn get_stream(&mut self) -> GtkEventStream {
        let mut core = self.core.lock().unwrap();

        // Create the stream core
        let stream_core = GtkEventStreamCore {
            dropped:    false,
            pending:    VecDeque::new(),
            poll_event: None
        };
        let stream_core = Arc::new(Mutex::new(stream_core));

        // Add to the list of streams attached to this event sink
        core.streams.push(Arc::clone(&stream_core));

        // Generate the new event stream
        GtkEventStream {
            core:       stream_core,
            sink_core:  Arc::clone(&self.core)
        }
    }
}

impl Sink for GtkEventSink {
    type SinkItem   = GtkEvent;
    type SinkError  = ();

    fn start_send(&mut self, item: GtkEvent) -> StartSend<GtkEvent, ()> {
        // Get the core
        let mut core = self.core.lock().unwrap();

        // Clean out any stream that is no longer active
        core.streams.retain(|stream_core| !stream_core.lock().unwrap().dropped);

        // If there are active streams, then post the event and wake any up that are waiting
        if core.streams.len() > 0 {
            for stream in core.streams.iter() {
                let mut stream_core = stream.lock().unwrap();

                // Post the event
                stream_core.pending.push_front(item.clone());

                // Wake the stream if it's asleep
                stream_core.poll_event.take().map(|task| task.notify());
            }
        }

        // Item went to the sink
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        let mut core = self.core.lock().unwrap();

        // Clean out any stream that is no longer active
        core.streams.retain(|stream_core| !stream_core.lock().unwrap().dropped);

        // Check for pending actions
        let any_pending = core.streams.iter()
            .any(|stream_core| stream_core.lock().unwrap().pending.len() > 0);

        if any_pending {
            // There are tasks waiting to be consumed
            core.poll_complete = Some(task::current());

            Ok(Async::NotReady)
        } else {
            // There are no tasks waiting to be consumed
            Ok(Async::Ready(()))
        }
    }
}

impl Stream for GtkEventStream {
    type Item   = GtkEvent;
    type Error  = ();

    fn poll(&mut self) -> Poll<Option<GtkEvent>, ()> {
        // If we need to signal the sink core, we'll need it locked, and it must always be locked ahead of the stream core
        let mut sink_core   = self.sink_core.lock().unwrap();
        let mut stream_core = self.core.lock().unwrap();

        // If there is a pending event, return it immediately
        if let Some(next_event) = stream_core.pending.pop_front() {
            if stream_core.pending.len() == 0 {
                // All events are clear: the sink can poll for completion again
                sink_core.poll_complete.take().map(|task| task.notify());
            }

            // Next event is ready
            Ok(Async::Ready(Some(next_event)))
        } else if sink_core.dropped {
            // The sink is no longer available to produce more events
            Ok(Async::Ready(None))
        } else {
            // No events are ready: wait for the next event to arrive
            stream_core.poll_event = Some(task::current());

            Ok(Async::NotReady)
        }
    }
}

impl Drop for GtkEventSink {
    fn drop(&mut self) {
        // Mark the core as dropped
        let mut core = self.core.lock().unwrap();
        core.dropped = true;

        // Wake all of the streams so they have a chance to signal that they are finished
        for stream_core in core.streams.iter() {
            stream_core.lock().unwrap().poll_event.take().map(|task| task.notify());
        }
    }
}

impl Drop for GtkEventStream {
    fn drop(&mut self) {
        let mut sink_core   = self.sink_core.lock().unwrap();
        let mut stream_core = self.core.lock().unwrap();

        // Mark the core as dropped
        stream_core.dropped = true;

        // Wake the sink to give it a chance to indicate that it has completed sending events
        sink_core.poll_complete.take().map(|task| task.notify());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::thread;
    use futures::executor;

    #[test]
    fn can_send_and_receive_an_event() {
        let mut sink    = GtkEventSink::new();
        let mut stream  = sink.get_stream();

        // Spawn a thread to send a message to the sink
        thread::spawn(move || {
            let mut sink_executor = executor::spawn(sink);
            sink_executor.wait_send(GtkEvent::None);
        });

        // Receive the event from the thread
        let mut stream_executor = executor::spawn(stream);

        let next_event = stream_executor.wait_stream();
        assert!(next_event == Some(Ok(GtkEvent::None)));
    }

    #[test]
    fn can_send_and_receive_an_event_to_multiple_streams() {
        let mut sink    = GtkEventSink::new();
        let mut stream1 = sink.get_stream();
        let mut stream2 = sink.get_stream();
        let mut stream3 = sink.get_stream();

        // Spawn a thread to send a message to the sink
        thread::spawn(move || {
            let mut sink_executor = executor::spawn(sink);
            sink_executor.wait_send(GtkEvent::None);
        });

        // Receive the event from the thread
        let mut stream_executor1 = executor::spawn(stream1);
        let mut stream_executor3 = executor::spawn(stream2);
        let mut stream_executor2 = executor::spawn(stream3);

        assert!(stream_executor1.wait_stream() == Some(Ok(GtkEvent::None)));
        assert!(stream_executor2.wait_stream() == Some(Ok(GtkEvent::None)));
        assert!(stream_executor3.wait_stream() == Some(Ok(GtkEvent::None)));

        assert!(stream_executor1.wait_stream() == None);
        assert!(stream_executor2.wait_stream() == None);
        assert!(stream_executor3.wait_stream() == None);
    }

    #[test]
    fn closing_sink_ends_stream() {
        let mut sink    = GtkEventSink::new();
        let mut stream  = sink.get_stream();

        // Spawn a thread to send a message to the sink
        thread::spawn(move || {
            let mut sink_executor = executor::spawn(sink);
            sink_executor.wait_send(GtkEvent::None);
        });

        // Receive the event from the thread
        let mut stream_executor = executor::spawn(stream);

        let _next_event = stream_executor.wait_stream();
        let last_event = stream_executor.wait_stream();

        assert!(last_event == None);
    }
}
