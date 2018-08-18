use super::log_msg::*;
use super::context::*;
use super::message::*;
use super::static_log::*;

use flo_stream::*;

use desync::*;
use futures::*;

use std::sync::*;

///
/// A log publisher provides a way to publish log messages to subscribers
/// 
pub struct LogPublisher {
    /// The context for this log
    context: Arc<Desync<LogContext>>,
}

impl LogPublisher {
    ///
    /// Creates a new log publisher
    /// 
    /// Log publishers will republish to the current thread logger by default
    /// 
    pub fn new() -> LogPublisher {
        let log_default = Self::new_empty();

        pipe_in(Arc::clone(&log_default.context), log_default.subscribe_default(), |_context, log_msg| {
            log_msg.map(|log_msg| log().log(log_msg)).ok();
        });

        log_default        
    }

    ///
    /// Creates a new log publisher with no default behaviour
    /// 
    pub (crate) fn new_empty() -> LogPublisher {
        LogPublisher {
            context: Arc::new(Desync::new(LogContext::new()))
        }
    }

    ///
    /// Sends a log message to the context
    /// 
    fn log_in_context(context: &mut LogContext, message: Log) {
        let num_subscribers = context.publisher.get_ref().count_subscribers();

        // Send to the subscribers of this log
        context.publisher.wait_send(message.clone()).unwrap();

        // Send to the parent or the default log
        if num_subscribers == 0 {
            context.default.as_mut().map(|default| default.wait_send(message).unwrap());
        }
    }

    ///
    /// Sends a message to the subscribers for this log
    /// 
    pub fn log<Msg: 'static+LogMessage>(&self, message: Msg) {
        self.context.sync(|context| {
            // Messages are delivered as Arc<Log>s to prevent them being copied around when there's a complicated hierarchy
            let message = Log::from(message);
            Self::log_in_context (context, message);
        });
    }

    ///
    /// Sends a stream of log messages to this log
    /// 
    pub fn stream<Msg: LogMessage, LogStream: 'static+Send+Stream<Item=Msg, Error=()>>(&self, stream: LogStream) {
        // Pipe the stream through to the context
        pipe_in(Arc::clone(&self.context), stream, |context, message| {
            if let Ok(message) = message {
                let message = Log::from(message);
                Self::log_in_context(context, message);
            }
        });
    }

    ///
    /// Subscribes to this log stream
    /// 
    pub fn subscribe(&self) -> impl Stream<Item=Log, Error=()> {
        self.context.sync(|context| context.publisher.subscribe())
    }

    ///
    /// Creates a 'default' subscriber for this log stream (messages will be sent here only if there are no other subscribers to this log)
    /// 
    pub fn subscribe_default(&self) -> impl Stream<Item=Log, Error=()> {
        self.context.sync(|context| {
            if context.default.is_none() {
                context.default = Some(executor::spawn(Publisher::new(100)));
            }

            context.default.as_mut().unwrap().subscribe()
        })
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
