use super::log::*;
use super::message::*;

use flo_stream::*;

use futures::*;
use futures::executor;
use futures::executor::Spawn;

use std::sync::*;

///
/// A log publisher provides a way to publish log messages to subscribers
/// 
pub struct LogPublisher {
    /// The pubsub publisher for this log
    publisher: Spawn<Publisher<Arc<Log>>>
}

impl LogPublisher {
    ///
    /// Creates a new log publisher
    /// 
    pub fn new() -> LogPublisher {
        LogPublisher {
            publisher: executor::spawn(Publisher::new(100))
        }
    }

    ///
    /// Sends a message to the subscribers for this log
    /// 
    pub fn log<Msg: LogMessage>(&mut self, message: Msg) {
        self.publisher.wait_send(Arc::new(Log::from(message))).unwrap()
    }

    ///
    /// Subscribes to this log stream
    /// 
    pub fn subscribe(&mut self) -> impl Stream<Item=Arc<Log>, Error=()> {
        self.publisher.subscribe()
    }
}
