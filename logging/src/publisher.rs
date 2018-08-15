use super::log::*;
use super::context::*;
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
    publisher: Spawn<Publisher<Arc<Log>>>,

    /// The context for this log
    context: Arc<Mutex<LogContext>>
}

impl LogPublisher {
    ///
    /// Creates a new log publisher
    /// 
    pub fn new() -> LogPublisher {
        LogPublisher {
            publisher:  executor::spawn(Publisher::new(100)),
            context:    Arc::new(Mutex::new(LogContext::new()))
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

///
/// Creates a new log publisher that tracks the same set of subscribers as the original
/// 
impl Clone for LogPublisher {
    fn clone(&self) -> LogPublisher {
        LogPublisher {
            publisher:  executor::spawn(self.publisher.get_ref().republish()),
            context:    Arc::clone(&self.context)
        }
    }
}