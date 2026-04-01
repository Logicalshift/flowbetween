use super::message::*;
use super::privilege::*;

use log;

use std::sync::*;
use std::collections::HashMap;

///
/// Structure that stores the data from a log message
///
#[derive(Clone, PartialEq, Debug)]
struct LogCore {
    message:    String,
    level:      log::Level,
    privilege:  LogPrivilege,
    fields:     HashMap<String, String>
}

///
/// Structure that stores a copy of the data from a log message
///
/// This stores the message data as a reference which makes it convenient for passing around via the publisher.
///
#[derive(Clone, PartialEq, Debug)]
pub struct LogMsg {
    core: Arc<LogCore>
}

impl LogMessage for LogMsg {
    fn message<'a>(&'a self) -> &'a str { &*self.core.message }

    fn level(&self) -> log::Level { self.core.level }

    fn privilege(&self) -> LogPrivilege { self.core.privilege }

    fn fields(&self) -> Vec<(String, String)> { self.core.fields.iter().map(|(a, b)| (a.clone(), b.clone())).collect() }
}

impl LogMsg {
    ///
    /// Creates a new Log from a log message
    ///
    pub fn from<Msg: LogMessage>(msg: Msg) -> LogMsg {
        let core = LogCore {
            message:    msg.message().to_string(),
            level:      msg.level(),
            privilege:  msg.privilege(),
            fields:     msg.fields().into_iter().collect()
        };

        LogMsg {
            core: Arc::new(core)
        }
    }

    ///
    /// Merges a set of fields into this log message
    ///
    /// If any fields in the new fields list are already set to a value, the original value is left in place
    ///
    pub fn merge_fields(&mut self, new_fields: &Vec<(String, String)>) {
        if new_fields.len() > 0 {
            // Create a replacement core
            let mut new_core = (*self.core).clone();

            // Merge in the new fields
            for (field_name, field_value) in new_fields {
                if !new_core.fields.contains_key(field_name) {
                    new_core.fields.insert(field_name.clone(), field_value.clone());
                }
            }

            // Store in this object, replacing the old core
            self.core = Arc::new(new_core);
        }
    }

    pub fn field_value<'a>(&'a self, field_name: &str) -> Option<&'a str> {
        self.core.fields
            .get(field_name)
            .map(|field_value| &**field_value)
    }
}
