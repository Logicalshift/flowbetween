use super::log::*;

use flo_stream::*;

use futures::executor::Spawn;

///
/// Represents the context of a publisher
/// 
pub struct LogContext {
    /// The parent of this context (all log messages are republished here)
    parent: Option<Spawn<Publisher<Log>>>,

    /// If there are no subscribers to a particular log, messages are sent here instead
    default: Option<Spawn<Publisher<Log>>>
}

impl LogContext {
    ///
    /// Creates a new LogContext
    /// 
    pub fn new() -> LogContext {
        LogContext {
            parent:     None,
            default:    None
        }
    }
}
