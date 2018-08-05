use super::subscriber::*;
use super::pubsub_core::*;

use futures::*;
use futures::sink::Sink;

use std::sync::*;
use std::collections::{HashMap, VecDeque};

///
/// A publisher represents a sink that sends messages to zero or more subscribers
/// 
pub struct Publisher<Message> {
    /// The next ID to assign to a new subscriber
    next_subscriber_id: usize,

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
            subscribers:    HashMap::new(),
            max_queue_size: buffer_size,
            notify_ready:   None
        };

        // Build the publisher itself
        Publisher {
            next_subscriber_id: 0,
            core:               Arc::new(Mutex::new(core))
        }
    }

    ///
    /// Subscribes to this publisher
    /// 
    /// Subscribers only receive messages sent to the publisher after they are created.
    /// 
    pub fn subscribe(&mut self) -> Subscriber<Message> {
        // Assign a subscriber ID
        let subscriber_id = self.next_subscriber_id;
        self.next_subscriber_id += 1;

        // Create the subscriber core
        let sub_core = SubCore {
            id:             subscriber_id,
            subscribed:     true,
            waiting:        VecDeque::new(),
            notify_waiting: None
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

impl<Message: Clone> Sink for Publisher<Message> {
    type SinkItem   = Message;
    type SinkError  = ();

    fn start_send(&mut self, item: Message) -> StartSend<Message, ()> {
        // Publish the message to the core
        let notify = self.core.lock().unwrap().publish(&item);

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
        unimplemented!()
    }
}
