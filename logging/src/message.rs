use super::level::*;
use super::privilege::*;

///
/// Trait implemented by items representing a log message
/// 
pub trait LogMessage {
    ///
    /// Returns a string representation of this log message
    /// 
    fn message(&self) -> String;

    ///
    /// Returns the verbosity/seriousness level of this log message
    /// 
    fn level(&self) -> LogLevel { LogLevel::Info }

    ///
    /// Returns the privilege level of this log message (who can read it) 
    ///
    fn privilege(&self) -> LogPrivilege { LogPrivilege::Application }

    ///
    /// Returns this log message formatted into a series of named fields
    /// 
    fn fields(&self) -> Vec<(String, String)> { vec![("message".to_string(), self.message())] }
}

impl LogMessage for String {
    fn message(&self) -> String { self.clone() }
}

impl<'a> LogMessage for &'a str {
    fn message(&self) -> String { self.to_string() }
}

impl<'a, Msg: LogMessage> LogMessage for (LogLevel, Msg) {
    fn message(&self) -> String { self.1.message() }

    fn level(&self) -> LogLevel { self.0 }

    fn privilege(&self) -> LogPrivilege { self.1.privilege() }

    fn fields(&self) -> Vec<(String, String)> { self.1.fields() }
}

impl<'a, Msg: LogMessage> LogMessage for (LogPrivilege, Msg) {
    fn message(&self) -> String { self.1.message() }

    fn level(&self) -> LogLevel { self.1.level() }

    fn privilege(&self) -> LogPrivilege { self.0 }

    fn fields(&self) -> Vec<(String, String)> { self.1.fields() }
}
