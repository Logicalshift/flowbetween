use super::super::gtk_thread::*;
use super::super::gtk_action::*;

use flo_stream::*;

use ::desync::*;
use futures::*;
use futures::task::{Poll};

use std::sync::*;

pub type GtkActionSink = Arc<Desync<WeakPublisher<Vec<GtkAction>>>>;

///
/// Publishes a list of actions to the specified action sink
///
pub fn publish_actions(sink: &GtkActionSink, actions: Vec<GtkAction>) {
    let _ = sink.future(move |sink| async move {
        sink.publish(actions).await
    }.boxed());
}

///
/// Returns a future that runs actions published to a publisher on a thread
///
pub async fn run_gtk_actions_for_thread(thread: Arc<GtkThread>, actions: WeakPublisher<Vec<GtkAction>>) {
    // Subscribe to the actions that are being published
    let mut actions = actions.subscribe();

    // Dispatch them to the thread as they arrive
    loop {
        let actions = read_actions(&mut actions).await;

        match actions {
            None            => { return; }
            Some(actions)   => { thread.perform_actions(actions) }
        }
    }
}

///
/// Waits for an event and then reads as many as possible from the queue
///
async fn read_actions(subscriber: &mut Subscriber<Vec<GtkAction>>) -> Option<Vec<GtkAction>> {
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

/*
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
*/