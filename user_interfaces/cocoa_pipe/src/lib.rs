//!
//! `cocoa_pipe` is a library that describes the actions and events of an application implemented
//! using Cocoa as enumerations of messages, and provides a means to convert the more generic events
//! generated in the `flo_ui` library into these events.
//!

#[macro_use] extern crate num_derive;

mod action;
mod event;
mod view_type;
mod ui_pipe;
mod regulator;
mod app_state;
mod view_state;
mod canvas_model;
mod actions_from;
mod actions_from_control_attribute;

pub use self::action::*;
pub use self::event::*;
pub use self::view_type::*;
pub use self::ui_pipe::*;
