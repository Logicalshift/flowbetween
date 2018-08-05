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
            core: Arc::new(Mutex::new(core))
        }
    }

    ///
    /// Subscribes to this publisher
    /// 
    /// Subscribers only receive messages sent to the publisher after they are created.
    /// 
    pub fn subscribe(&mut self) -> Subscriber<Message> {
        unimplemented!()
    }
}

impl<Message: Clone> Sink for Publisher<Message> {
    type SinkItem   = Message;
    type SinkError  = ();

    fn start_send(&mut self, item: Message) -> StartSend<Message, ()> {
        unimplemented!()
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        unimplemented!()
    }
}
