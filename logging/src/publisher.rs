use super::log::*;
use super::context::*;
use super::message::*;

use flo_stream::*;

use desync::*;
use futures::*;

use std::sync::*;

///
/// A log publisher provides a way to publish log messages to subscribers
/// 
pub struct LogPublisher {
    /// The context for this log
    context: Arc<Desync<LogContext>>
}

impl LogPublisher {
    ///
    /// Creates a new log publisher
    /// 
    pub fn new() -> LogPublisher {
        LogPublisher {
            context: Arc::new(Desync::new(LogContext::new()))
        }
    }

    ///
    /// Sends a log message to the context
    /// 
    fn log_in_context(context: &mut LogContext, message: Arc<Log>) {
        let num_subscribers = context.publisher.get_ref().count_subscribers();

        // Send to the subscribers of this log
        context.publisher.wait_send(Arc::clone(&message)).unwrap();

        // Send to the parent or the default log
        if num_subscribers == 0 {
            context.default.as_mut().map(|default| default.wait_send(Arc::clone(&message)).unwrap());
        }
    }

    ///
    /// Sends a message to the subscribers for this log
    /// 
    pub fn log<Msg: 'static+LogMessage>(&self, message: Msg) {
        self.context.sync(|context| {
            // Messages are delivered as Arc<Log>s to prevent them being copied around when there's a complicated hierarchy
            let message = Arc::new(Log::from(message));
            Self::log_in_context(context, message);
        });
    }

    ///
    /// Sends a stream of log messages to this log
    /// 
    pub fn stream<Msg: LogMessage, LogStream: 'static+Send+Stream<Item=Msg, Error=()>>(&self, stream: LogStream) {
        // Pipe the stream through to the context
        pipe_in(Arc::clone(&self.context), stream, |context, message| {
            if let Ok(message) = message {
                let message = Arc::new(Log::from(message));
                Self::log_in_context(context, message);
            }
        });
    }

    ///
    /// Subscribes to this log stream
    /// 
    pub fn subscribe(&self) -> impl Stream<Item=Arc<Log>, Error=()> {
        self.context.sync(|context| context.publisher.subscribe())
    }
}

///
/// Creates a new log publisher that tracks the same set of subscribers as the original
/// 
impl Clone for LogPublisher {
    fn clone(&self) -> LogPublisher {
        LogPublisher {
            context:    Arc::clone(&self.context)
        }
    }
}