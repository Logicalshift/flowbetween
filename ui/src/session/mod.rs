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
use super::controller::*;
use super::user_interface::*;
use binding::*;
use std::sync::*;

/// The basic user interface implemented by a UI session 
pub trait CoreUserInterface : UserInterface<Vec<UiEvent>, Vec<UiUpdate>, (), EventSink=UiEventSink, UpdateStream=UiUpdateStream> {
    type CoreController: Controller;

    /// Retrieves the control tree for this UI
    fn ui_tree(&self) -> BindRef<Control>;

    /// Retrieves the controler for this UI
    fn controller(&self) -> Arc<Self::CoreController>;
}
