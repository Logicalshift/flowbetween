//!
//! Provides convenience actions for anything implementing the Animation trait (generally generating
//! a sequence of edit actions for performing another particular action)
//!

mod edit_action;
mod motion_actions;

pub use self::edit_action::*;
pub use self::motion_actions::*;
