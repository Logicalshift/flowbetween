use super::log_msg::*;

use flo_stream::*;

use futures::executor;
use futures::executor::Spawn;

///
/// Represents the context of a publisher
///
pub struct LogContext {
    /// Where messages for this context should be published
    pub (crate) publisher: Spawn<Publisher<LogMsg>>,

    /// If there are no subscribers to a particular log, messages are sent here instead
    pub (crate) default: Option<Spawn<Publisher<LogMsg>>>,

    /// The fields to add to log messages sent to this context
    pub (crate) fields: Vec<(String, String)>
}

impl LogContext {
    ///
    /// Creates a new LogContext
    ///
    pub fn new() -> LogContext {
        LogContext {
            publisher:  executor::spawn(Publisher::new(100)),
            default:    None,
            fields:     vec![]
        }
    }
}
