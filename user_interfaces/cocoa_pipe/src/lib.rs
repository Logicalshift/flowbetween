//!
//! `cocoa_pipe` is a library that describes the actions and events of an application implemented
//! using Cocoa as enumerations of messages, and provides a means to convert the more generic events
//! generated in the `flo_ui` library into these events.
//!

mod cocoa_action;
mod cocoa_event;

pub use self::cocoa_action::*;
pub use self::cocoa_event::*;
