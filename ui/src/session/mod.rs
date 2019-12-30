mod event;
mod update;
mod core;
mod event_publisher;
mod update_stream;
pub mod state;
pub mod session;
mod canvas_stream;
mod viewmodel_stream;

pub use self::event::*;
pub use self::update::*;
pub use self::session::*;
pub use self::event_publisher::*;
pub use self::canvas_stream::*;
pub use self::viewmodel_stream::*;
pub use self::update_stream::*;

#[cfg(test)] mod tests;

use super::control::*;
use super::controller::*;
use super::user_interface::*;
use flo_binding::*;
use std::sync::*;

/// The basic user interface implemented by a UI session
pub trait CoreUserInterface : UserInterface<Vec<UiEvent>, Vec<UiUpdate>, (), UpdateStream=UiUpdateStream> {
    type CoreController: Controller;

    /// Retrieves the control tree for this UI
    fn ui_tree(&self) -> BindRef<Control>;

    /// Retrieves the controler for this UI
    fn controller(&self) -> Arc<Self::CoreController>;
}
