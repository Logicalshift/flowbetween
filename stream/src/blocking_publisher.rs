use super::publisher::*;
use super::subscriber::*;
use super::publisher_sink::*;

use futures::*;
use futures::task;
use futures::task::Task;
use futures::future;
use futures::future::Either;
use futures::sync::oneshot;

///
/// A blocking publisher is a publisher that blocks messages until it has enough subscribers
/// 
/// This is useful for cases where a publisher is being used asynchronously and wants to ensure that
/// its messages are sent to at least one subscriber. Once the required number of subscribers is
/// reached, this will not block again even if some subscribers are dropped.
/// 
pub struct BlockingPublisher<Message> {
    /// True if there are not currently enouogh subscribers in this publisher
    insufficient_subscribers: bool,

    /// The number of required subscribers
    required_subscribers: usize,

    /// The publisher where messages will be relayed
    publisher: Publisher<Message>,

    /// Notification to be sent when there are enough subscribers in this publisher
    notify_full: Option<Task>,

    /// Futures to be notified when there are enough subscribers for this publisher
    notify_futures: Vec<oneshot::Sender<()>>
}

impl<Message: Clone> BlockingPublisher<Message> {
    ///
    /// Creates a new blocking publisher
    /// 
    /// This publisher will refuse to receive any items until at least required_subscribers are connected.
    /// The buffer size indicates the number of queued items permitted per buffer.
    /// 
    pub fn new(required_subscribers: usize, buffer_size: usize) -> BlockingPublisher<Message> {
        BlockingPublisher {
            insufficient_subscribers:   required_subscribers != 0,
            required_subscribers:       required_subscribers,
            publisher:                  Publisher::new(buffer_size),
            notify_full:                None,
            notify_futures:             vec![]
        }
    }

    ///
    /// Returns a future that will complete when this publisher has enough subscribers
    /// 
    /// This is useful as a way to avoid blocking with `wait_send` when setting up the publisher
    /// 
    pub fn when_ready(&mut self) -> impl Future<Item=(), Error=Canceled> {
        if self.insufficient_subscribers {
            // Return a future that will be notified when we have enough subscribers
            let (sender, receiver) = oneshot::channel();

            // Notify when there are enough subscribers
            self.notify_futures.push(sender);

            Either::A(receiver)
        } else {
            // Already ready
            Either::B(future::ok(()))
        }
    }
}

impl<Message: Clone> PublisherSink<Message> for BlockingPublisher<Message> {
    fn subscribe(&mut self) -> Subscriber<Message> {
        // Create the subscription
        let subscription = self.publisher.subscribe();

        // Wake the sink if we get enough subscribers
        if self.insufficient_subscribers && self.publisher.count_subscribers() >= self.required_subscribers {
            // We now have enough subscribers
            self.insufficient_subscribers = false;
            
            // Notify anything that is blocking on this publisher
            self.notify_full.take().map(|notify| notify.notify());

            // Mark any futures that are waiting on this publisher
            self.notify_futures.drain(..)
                .for_each(|sender| { sender.send(()).ok(); });
        }

        // Result is our new subscription
        subscription
    }
}

impl<Message: Clone> Sink for BlockingPublisher<Message> {
    type SinkItem   = Message;
    type SinkError  = ();

    fn start_send(&mut self, item: Message) -> StartSend<Message, ()> {
        if self.insufficient_subscribers {
            // Not enough subscribers, so refuse to send
            self.notify_full = Some(task::current());
            Ok(AsyncSink::NotReady(item))
        } else {
            // Have reached the required number of subscribers, so pass through
            self.publisher.start_send(item)
        }
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        if self.insufficient_subscribers {
            // Not enough subscribers, so refuse to send
            self.notify_full = Some(task::current());
            Ok(Async::NotReady)
        } else {
            // Have reached the required number of subscribers, so pass through to the main publisher
            self.publisher.poll_complete()
        }
    }
}