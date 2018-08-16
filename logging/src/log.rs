use super::level::*;
use super::message::*;
use super::privilege::*;

use std::sync::*;

///
/// Structure that stores the data from a log message
/// 
#[derive(Clone, PartialEq, Debug)]
struct LogCore {
    message:    String,
    level:      LogLevel,
    privilege:  LogPrivilege,
    fields:     Vec<(String, String)>
}

///
/// Structure that stores a copy of the data from a log message
/// 
/// This stores the message data as a reference which makes it convenient for passing around via the publisher.
/// 
#[derive(Clone, PartialEq, Debug)]
pub struct Log {
    core: Arc<LogCore>
}

impl LogMessage for Log {
    fn message(&self) -> String { self.core.message.clone() }

    fn level(&self) -> LogLevel { self.core.level }

    fn privilege(&self) -> LogPrivilege { self.core.privilege }

    fn fields(&self) -> Vec<(String, String)> { self.core.fields.clone() }
}

impl Log {
    ///
    /// Creates a new Log from a log message
    /// 
    pub fn from<Msg: LogMessage>(msg: Msg) -> Log {
        let core = LogCore {
            message:    msg.message(),
            level:      msg.level(),
            privilege:  msg.privilege(),
            fields:     msg.fields()
        };

        Log {
            core: Arc::new(core)
        }
    }
}
