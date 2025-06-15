//!
//! This provides an implementation of Flowbetween's dialog messages implemented using egui.
//!
//! The 'subprograms that send messages' design of flo_scene is in some respects quite similar to how imguis work,
//! and egui returns render requests rather than being the interface with the OS, which also suits FlowBetween's
//! design. However, event handling is done by sending messages in FlowBetween, which is quite different from how
//! imguis traditionally work.
//!

mod dialog_egui;
mod key;
mod events;
mod draw;
mod hub;
mod state;

pub use hub::*;
