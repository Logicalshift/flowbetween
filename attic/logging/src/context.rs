use super::log_msg::*;

use flo_stream::*;

///
/// Represents the context of a publisher
///
pub struct LogContext {
    /// Where messages for this context should be published
    pub (crate) publisher: Publisher<LogMsg>,

    /// If there are no subscribers to a particular log, messages are sent here instead
    pub (crate) default: Option<Publisher<LogMsg>>,

    /// The fields to add to log messages sent to this context
    pub (crate) fields: Vec<(String, String)>
}

impl LogContext {
    ///
    /// Creates a new LogContext
    ///
    pub fn new() -> LogContext {
        LogContext {
            publisher:  Publisher::new(100),
            default:    None,
            fields:     vec![]
        }
    }
}
