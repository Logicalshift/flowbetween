use super::pubsub_core::*;

use futures::*;
use futures::task;

use std::sync::*;

///
/// Represents a subscriber stream from a publisher sink
/// 
pub struct Subscriber<Message> {
    /// The publisher core (shared between all subscribers)
    /// 
    /// Note that when locking the pub_core must always be locked first (if it needs to be locked)
    pub_core: Weak<Mutex<PubCore<Message>>>,

    /// The subscriber core (used only by this subscriber)
    /// 
    /// Note that when locking the pub_core must always be locked first (if it needs to be locked)
    sub_core: Arc<Mutex<SubCore<Message>>>
}

impl<Message> Subscriber<Message> {
    ///
    /// Creates a new subscriber
    /// 
    pub (crate) fn new(pub_core: Weak<Mutex<PubCore<Message>>>, sub_core: Arc<Mutex<SubCore<Message>>>) -> Subscriber<Message> {
        Subscriber {
            pub_core,
            sub_core
        }
    }
}

impl<Message> Stream for Subscriber<Message> {
    type Item   = Message;
    type Error  = ();

    fn poll(&mut self) -> Poll<Option<Message>, ()> {
        // TODO: notify the publisher if necessary

        // Try to read a message from the waiting list
        let mut sub_core    = self.sub_core.lock().unwrap();
        let next_message    = sub_core.waiting.pop_front();

        if let Some(next_message) = next_message {
            // Return the next message if it's available
            Ok(Async::Ready(Some(next_message)))
        } else if self.pub_core.upgrade().is_none() {
            // Stream has finished if the publisher core is no longer available
            Ok(Async::Ready(None))
        } else {
            // If the publisher is still alive and there are no messages available, store notification and carry on
            sub_core.notify_waiting = Some(task::current());
            Ok(Async::NotReady)
        }
    }
}
