use super::log::*;

use flo_stream::*;

use futures::executor::Spawn;

use std::sync::*;

///
/// Represents the context of a publisher
/// 
pub struct LogContext {
    /// The parent of this context (all log messages are republished here)
    pub (crate) parent: Option<Spawn<Publisher<Arc<Log>>>>,

    /// If there are no subscribers to a particular log, messages are sent here instead
    pub (crate) default: Option<Spawn<Publisher<Arc<Log>>>>
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
