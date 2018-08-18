use super::log_msg::*;

use flo_stream::*;

use futures::executor;
use futures::executor::Spawn;

///
/// Represents the context of a publisher
/// 
pub struct LogContext {
    /// Where messages for this context should be published
    pub (crate) publisher: Spawn<Publisher<Log>>,

    /// If there are no subscribers to a particular log, messages are sent here instead
    pub (crate) default: Option<Spawn<Publisher<Log>>>
}

impl LogContext {
    ///
    /// Creates a new LogContext
    /// 
    pub fn new() -> LogContext {
        LogContext {
            publisher:  executor::spawn(Publisher::new(100)),
            default:    None
        }
    }
}
