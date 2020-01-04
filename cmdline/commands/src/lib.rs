//!
//! # flo_commands
//! 
//! This provides a stream-based API for issuing scripting commands for opening and
//! editing a FlowBetween animation.
//!

mod command;
mod command_runner;
mod output;
mod char_output;
mod subcommands;

pub use self::command::*;
pub use self::command_runner::*;
pub use self::output::*;
pub use self::char_output::*;
