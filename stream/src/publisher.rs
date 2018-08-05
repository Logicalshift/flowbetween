use super::subscriber::*;
use super::pubsub_core::*;
use super::publisher_sink::*;

use futures::*;
use futures::sink::Sink;

use std::sync::*;
use std::collections::{HashMap, VecDeque};

///
/// A publisher represents a sink that sends messages to zero or more subscribers
/// 
/// Call `subscribe()` to create subscribers. Any messages sent to this sink will be relayed to all connected
/// subscribers. If the publisher is dropped, any connected subscribers will relay all sent messages and then
/// indicate that they have finished.
/// 
pub struct Publisher<Message> {
    /// The shared core of this publisher
    core: Arc<Mutex<PubCore<Message>>>
}

impl<Message: Clone> Publisher<Message> {
    ///
    /// Creates a new publisher with a particular buffer size
    /// 
    pub fn new(buffer_size: usize) -> Publisher<Message> {
        // Create the core
        let core = PubCore {
            next_subscriber_id: 0,
            subscribers:        HashMap::new(),
            max_queue_size:     buffer_size
        };

        // Build the publisher itself
        Publisher {
            core:               Arc::new(Mutex::new(core))
        }
    }

    ///
    /// Counts the number of subscribers in this publisher
    /// 
    pub fn count_subscribers(&self) -> usize {
        self.core.lock().unwrap().subscribers.len()
    }
}

impl<Message: Clone> PublisherSink<Message> for Publisher<Message> {
    ///
    /// Subscribes to this publisher
    /// 
    /// Subscribers only receive messages sent to the publisher after they are created.
    /// 
    fn subscribe(&mut self) -> Subscriber<Message> {
        // Assign a subscriber ID
        let subscriber_id = {
            let mut core    = self.core.lock().unwrap();
            let id          = core.next_subscriber_id;
            core.next_subscriber_id += 1;

            id
        };

        // Create the subscriber core
        let sub_core = SubCore {
            id:                 subscriber_id,
            published:          true,
            waiting:            VecDeque::new(),
            notify_waiting:     None,
            notify_ready:       None,
            notify_complete:    None
        };

        // The new subscriber needs a reference to the sub_core and the pub_core
        let sub_core = Arc::new(Mutex::new(sub_core));
        let pub_core = Arc::downgrade(&self.core);

        // Register the subscriber with the core, so it will start receiving messages
        {
            let mut core = self.core.lock().unwrap();
            core.subscribers.insert(subscriber_id, Arc::clone(&sub_core));
        }

        // Create the subscriber
        Subscriber::new(pub_core, sub_core)
    }
}

impl<Message> Drop for Publisher<Message> {
    fn drop(&mut self) {
        let to_notify = {
            // Lock the core
            let pub_core = self.core.lock().unwrap();

            // Mark all the subscribers as unpublished and notify them so that they close
            let mut to_notify = vec![];

            for mut subscriber in pub_core.subscribers.values() {
                let mut subscriber = subscriber.lock().unwrap();

                // Unpublish the subscriber (so that it hits the end of the stream)
                subscriber.published    = false;
                subscriber.notify_ready = None;

                // Add to the things to notify once the lock is released
                to_notify.push(subscriber.notify_waiting.take());
            }

            // Return the notifications outside of the lock
            to_notify
        };

        // Notify any subscribers that are waiting that we're unpublished
        to_notify.into_iter().filter_map(|notify| notify).for_each(|notify| notify.notify());
    }
}

impl<Message: Clone> Sink for Publisher<Message> {
    type SinkItem   = Message;
    type SinkError  = ();

    fn start_send(&mut self, item: Message) -> StartSend<Message, ()> {
        // Publish the message to the core
        let notify = { self.core.lock().unwrap().publish(&item) };

        if let Some(notify) = notify {
            // Notify all the subscribers that the item has been published
            notify.into_iter().for_each(|notify| notify.notify());

            // Message sent
            Ok(AsyncSink::Ready)
        } else {
            // At least one subscriber has a full queue, so the message could not be sent
            Ok(AsyncSink::NotReady(item))
        }
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        if self.core.lock().unwrap().complete() {
            // All subscribers are ready to receive a message
            Ok(Async::Ready(()))
        } else {
            // At least one subscriber has a full buffer
            Ok(Async::NotReady)
        }
    }
}
