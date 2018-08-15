use super::level::*;
use super::message::*;
use super::privilege::*;

///
/// Structure that stores the data from a log message
/// 
#[derive(Clone, PartialEq, Debug)]
pub struct Log {
    message:    String,
    level:      LogLevel,
    privilege:  LogPrivilege,
    fields:     Vec<(String, String)>
}

impl LogMessage for Log {
    fn message(&self) -> String { self.message.clone() }

    fn level(&self) -> LogLevel { self.level }

    fn privilege(&self) -> LogPrivilege { self.privilege }

    fn fields(&self) -> Vec<(String, String)> { self.fields.clone() }
}

impl Log {
    ///
    /// Creates a new Log from a log message
    /// 
    pub fn from<Msg: LogMessage>(msg: Msg) -> Log {
        Log {
            message:    msg.message(),
            level:      msg.level(),
            privilege:  msg.privilege(),
            fields:     msg.fields()
        }
    }
}
