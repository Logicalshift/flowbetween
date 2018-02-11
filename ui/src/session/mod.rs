mod event;
mod update;
mod core;
mod event_sink;
mod update_stream;
pub mod canvas_state;
pub mod state;
pub mod session;

pub use self::event::*;
pub use self::update::*;
pub use self::session::*;
pub use self::event_sink::*;
pub use self::update_stream::*;

#[cfg(test)] mod tests;

use super::control::*;
use super::user_interface::*;
use binding::*;

/// The basic user interface implemented by a UI session 
pub trait CoreUserInterface : UserInterface<UiEvent, Vec<UiUpdate>, (), EventSink=UiEventSink, UpdateStream=UiUpdateStream> {
    /// Retrieves the control tree for this UI
    fn ui_tree(&self) -> BindRef<Control>;
}
