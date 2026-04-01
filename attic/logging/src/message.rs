use super::privilege::*;

use std::sync::*;
use std::collections::HashMap;

use log;

///
/// Trait implemented by items representing a log message
///
pub trait LogMessage : Send {
    ///
    /// Returns a string representation of this log message
    ///
    fn message<'a>(&'a self) -> &'a str;

    ///
    /// Returns the verbosity/seriousness level of this log message
    ///
    fn level(&self) -> log::Level { log::Level::Info }

    ///
    /// Returns the privilege level of this log message (who can read it)
    ///
    fn privilege(&self) -> LogPrivilege { LogPrivilege::Application }

    ///
    /// Returns this log message formatted into a series of named fields
    ///
    fn fields(&self) -> Vec<(String, String)> { vec![("message".to_string(), self.message().to_string())] }
}

impl LogMessage for String {
    fn message<'a>(&'a self) -> &'a str { &*self }
}

impl<'a> LogMessage for &'a str {
    fn message<'b>(&'b self) -> &'b str { self }
}

impl<Msg: LogMessage> LogMessage for (log::Level, Msg) {
    fn message<'a>(&'a self) -> &'a str { self.1.message() }

    fn level(&self) -> log::Level { self.0 }

    fn privilege(&self) -> LogPrivilege { self.1.privilege() }

    fn fields(&self) -> Vec<(String, String)> { self.1.fields() }
}

impl<Msg: LogMessage> LogMessage for (LogPrivilege, Msg) {
    fn message<'a>(&'a self) -> &'a str { self.1.message() }

    fn level(&self) -> log::Level { self.1.level() }

    fn privilege(&self) -> LogPrivilege { self.0 }

    fn fields(&self) -> Vec<(String, String)> { self.1.fields() }
}

impl LogMessage for HashMap<String, String> {
    fn message<'a>(&'a self) -> &'a str { self.get("message").map(|msg| &**msg).unwrap_or("") }

    fn fields(&self) -> Vec<(String, String)> { self.iter().map(|(a, b)| (a.clone(), b.clone())).collect() }
}

impl<'a> LogMessage for HashMap<&'a str, &'a str> {
    fn message<'b>(&'b self) -> &'b str { self.get("message").map(|msg| *msg).unwrap_or("") }

    fn fields(&self) -> Vec<(String, String)> { self.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect() }
}

impl<Msg: Send+Sync+LogMessage> LogMessage for Arc<Msg> {
    fn message<'a>(&'a self) -> &'a str { (**self).message() }

    fn level(&self) -> log::Level { (**self).level() }

    fn privilege(&self) -> LogPrivilege { (**self).privilege() }

    fn fields(&self) -> Vec<(String, String)> { (**self).fields() }
}
