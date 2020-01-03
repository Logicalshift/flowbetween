use super::super::gtk_event::*;

use flo_stream::*;

use ::desync::*;
use futures::prelude::*;

use std::sync::*;

type GtkEventSink = Arc<Desync<WeakPublisher<GtkEvent>>>;

///
/// Sends an event immediately to an event sink
///
pub fn publish_event(sink: &GtkEventSink, event: GtkEvent) {
    let _ = sink.future(move |publisher| async move {
        publisher.publish(event).await
    }.boxed());
}

/*
///
/// Runs the main gtk event loop
///
pub (crate) async fn gtk_run_loop(gtk_events: WeakPublisher<Vec<GtkEvent>>) {
    // Subscribe to the events
    let gtk_events = gtk_events.subscribe();

    loop {
        // Read as many events as we can before we start to process them
        let events = read_events(&mut gtk_events);
    }
}

///
/// Waits for an event and then reads as many as possible from the queue
///
async fn read_events(subscriber: &mut Subscriber<Vec<GtkEvent>>) -> Option<Vec<GtkEvent>> {
    // Await the first event
    let events = subscriber.next().await;
    if events.is_none() {
        return None;
    }

    // Read any other waiting events from the subscriber
    let mut poll_events     = events;
    let mut poll_subscriber = Some(subscriber);

    let (events, returned_subscriber) = future::poll_fn(move |context| {
        // Add as many extra events as we can retrieve
        while let Poll::Ready(Some(more_events)) = poll_subscriber.as_mut().unwrap().poll_next_unpin(context) {
            poll_events.as_mut().unwrap().extend(more_events)
        }

        // Return the events (and the subscriber) to the sender
        Poll::Ready((poll_events.take(), poll_subscriber.take()))
    }).await;

    // Return the events that we retrieved
    subscriber = returned_subscriber.unwrap();
    events
}
*/

/*
///
/// Core data for a Gtk event sink or stream
///
struct GtkEventSinkCore {
    /// Number of event sinks referencing this core
    sink_count: usize,

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
            sink_count:     1,
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
    pub fn get_stream(&self) -> GtkEventStream {
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
                stream_core.pending.push_back(item.clone());

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
        } else if sink_core.sink_count <= 0 {
            // The sink is no longer available to produce more events
            Ok(Async::Ready(None))
        } else {
            // No events are ready: wait for the next event to arrive
            stream_core.poll_event = Some(task::current());

            Ok(Async::NotReady)
        }
    }
}

impl Clone for GtkEventSink {
    fn clone(&self) -> GtkEventSink {
        // Increase the reference count for the event sinks
        {
            let mut core = self.core.lock().unwrap();
            core.sink_count += 1;
        }

        // Generate a new event sink with the same core as this one
        GtkEventSink {
            core: Arc::clone(&self.core)
        }
    }
}

impl Drop for GtkEventSink {
    fn drop(&mut self) {
        // Mark the core as dropped
        let mut core = self.core.lock().unwrap();
        core.sink_count -= 1;

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
        let stream      = sink.get_stream();

        // Spawn a thread to send a message to the sink
        thread::spawn(move || {
            let mut sink_executor = executor::spawn(sink);
            sink_executor.wait_send(GtkEvent::None).unwrap();
        });

        // Receive the event from the thread
        let mut stream_executor = executor::spawn(stream);

        let next_event = stream_executor.wait_stream();
        assert!(next_event == Some(Ok(GtkEvent::None)));
    }

    #[test]
    fn can_send_and_receive_an_event_to_multiple_streams() {
        let mut sink    = GtkEventSink::new();
        let stream1     = sink.get_stream();
        let stream2     = sink.get_stream();
        let stream3     = sink.get_stream();

        // Spawn a thread to send a message to the sink
        thread::spawn(move || {
            let mut sink_executor = executor::spawn(sink);
            sink_executor.wait_send(GtkEvent::None).unwrap();
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
    fn can_send_event_from_multiple_sinks() {
        let mut sink    = Some(GtkEventSink::new());
        let stream      = sink.as_mut().unwrap().get_stream();

        for sink_num in 0..3 {
            let sink = sink.clone().unwrap();
            // Spawn a thread to send a message to the sink
            thread::spawn(move || {
                let mut sink_executor = executor::spawn(sink);
                sink_executor.wait_send(GtkEvent::None).unwrap();
            });
        }

        // Make sure the sink is dropped when all of the threads finish
        sink = None;

        // Receive the event from the thread
        let mut stream_executor = executor::spawn(stream);

        let next_event = stream_executor.wait_stream();
        assert!(next_event == Some(Ok(GtkEvent::None)));
        let next_event = stream_executor.wait_stream();
        assert!(next_event == Some(Ok(GtkEvent::None)));
        let next_event = stream_executor.wait_stream();
        assert!(next_event == Some(Ok(GtkEvent::None)));
        let next_event = stream_executor.wait_stream();
        assert!(next_event == None);
    }

    #[test]
    fn closing_sink_ends_stream() {
        let mut sink    = GtkEventSink::new();
        let stream      = sink.get_stream();

        // Spawn a thread to send a message to the sink
        thread::spawn(move || {
            let mut sink_executor = executor::spawn(sink);
            sink_executor.wait_send(GtkEvent::None).unwrap();
        });

        // Receive the event from the thread
        let mut stream_executor = executor::spawn(stream);

        let _next_event = stream_executor.wait_stream();
        let last_event = stream_executor.wait_stream();

        assert!(last_event == None);
    }
}
*/