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

impl<Message> Drop for Subscriber<Message> {
    fn drop(&mut self) {
        let notify = {
            // Lock the publisher and subscriber cores (note that the publisher core must always be locked first)
            let pub_core = self.pub_core.upgrade();

            if let Some(pub_core) = pub_core {
                // Lock the cores
                let mut pub_core = pub_core.lock().unwrap();
                let mut sub_core = self.sub_core.lock().unwrap();

                // Remove this subscriber from the publisher core
                pub_core.subscribers.remove(&sub_core.id);

                // Need to notify the core if it's waiting on this subscriber (might now be unblocked)
                sub_core.notify_ready.take()
            } else {
                // Need to notify the core if it's waiting on this subscriber (might now be unblocked)
                let mut sub_core = self.sub_core.lock().unwrap();
                sub_core.notify_ready.take()
            }
        };

        // After releasing the locks, notify the publisher if it's waiting on this subscriber
        if let Some(notify) = notify {
            notify.notify()
        }
    }
}

impl<Message> Stream for Subscriber<Message> {
    type Item   = Message;
    type Error  = ();

    fn poll(&mut self) -> Poll<Option<Message>, ()> {
        let (result, notify) = {
            // Try to read a message from the waiting list
            let mut sub_core    = self.sub_core.lock().unwrap();
            let next_message    = sub_core.waiting.pop_front();

            if let Some(next_message) = next_message {
                // Return the next message if it's available
                (Ok(Async::Ready(Some(next_message))), sub_core.notify_ready.take())
            } else if !sub_core.published {
                // Stream has finished if the publisher core is no longer available
                (Ok(Async::Ready(None)), None)
            } else {
                // If the publisher is still alive and there are no messages available, store notification and carry on
                sub_core.notify_waiting = Some(task::current());
                (Ok(Async::NotReady), None)
            }
        };

        // If there's something to notify as a result of this request, do so (note that we do this after releasing the core lock)
        if let Some(notify) = notify {
            notify.notify();
        }

        // Return the result
        result
    }
}
