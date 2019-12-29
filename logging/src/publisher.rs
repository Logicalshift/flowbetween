use super::log_msg::*;
use super::context::*;
use super::message::*;
use super::static_log::*;

use flo_stream::*;

use desync::{Desync, pipe_in};
use futures::future::{FutureExt};
use futures::prelude::*;

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
    /// Log publishers will republish to the current thread logger by default. Messages that originate from
    /// this publisher will have the target field added to it
    /// 
    pub fn new(target: &str) -> LogPublisher {
        // Create an empty publisher
        let log_default = Self::new_empty(vec![("target", target)]);

        // Messages are piped to the current logger if there is one
        let parent_log = current_log();
        parent_log.stream(log_default.subscribe_default());

        log_default
    }

    ///
    /// Cretes a new log publisher that will set some field values on messages before publishing them
    /// 
    /// Field values set this way are only specified 
    /// 
    pub fn new_with_fields<'a, FieldIter: 'a+IntoIterator<Item=(&'a str, &'a str)>>(target: &str, fields: FieldIter) -> LogPublisher {
        // Create a new logger with this target
        let logger = Self::new(target);

        // Extend the set of fields in its context
        let fields = fields.into_iter()
            .map(|(field_name, field_value)| (field_name.to_string(), field_value.to_string()))
            .collect::<Vec<_>>();
        logger.context.sync(move |context| context.fields.extend(fields));

        logger
    }

    ///
    /// Creates a new log publisher with no default behaviour
    /// 
    pub (crate) fn new_empty<'a, FieldIter: 'a+IntoIterator<Item=(&'a str, &'a str)>>(fields: FieldIter) -> LogPublisher {
        let logger = LogPublisher {
            context: Arc::new(Desync::new(LogContext::new()))
        };

        // Extend the set of fields in its context
        let fields = fields.into_iter()
            .map(|(field_name, field_value)| (field_name.to_string(), field_value.to_string()))
            .collect::<Vec<_>>();
        logger.context.sync(move |context| context.fields.extend(fields));

        logger
    }

    ///
    /// Sends a log message to the context
    /// 
    fn log_in_context(context: &mut LogContext, mut message: LogMsg) -> impl Future<Output=()> {
        let num_subscribers = context.publisher.count_subscribers();

        // Make sure that all the log fields are set properly
        message.merge_fields(&context.fields);

        // Send to the subscribers of this log
        let publish = context.publisher.publish(message.clone());

        // Send to the parent or the default log
        if num_subscribers == 0 {
            if let Some(default) = context.default.as_mut() {
                let default_publish = default.publish(message);

                publish.then(move |()| default_publish)
                    .right_future()
            } else {
                publish.left_future()
            }
        } else {
            publish.left_future()
        }
    }

    ///
    /// Sends a message to the subscribers for this log
    /// 
    pub fn log<Msg: LogMessage>(&self, message: Msg) {
        self.context.sync(|context| {
            // Messages are delivered as Arc<Log>s to prevent them being copied around when there's a complicated hierarchy
            let message = LogMsg::from(message);
            Self::log_in_context(context, message);
        });
    }

    ///
    /// Sends a stream of log messages to this log
    /// 
    pub fn stream<Msg: LogMessage, LogStream: 'static+Unpin+Send+Stream<Item=Msg>>(&self, stream: LogStream) {
        // Pipe the stream through to the context
        pipe_in(Arc::clone(&self.context), stream, |context, message| {
            let message = LogMsg::from(message);
            Self::log_in_context(context, message);
        });
    }

    ///
    /// Subscribes to this log stream
    /// 
    pub fn subscribe(&self) -> impl Stream<Item=LogMsg> {
        self.context.sync(|context| context.publisher.subscribe())
    }

    ///
    /// Creates a 'default' subscriber for this log stream (messages will be sent here only if there are no other subscribers to this log)
    /// 
    pub fn subscribe_default(&self) -> impl Stream<Item=LogMsg> {
        self.context.sync(|context| {
            if context.default.is_none() {
                context.default = Some(Publisher::new(100));
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
