use super::action_sink::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;
use super::super::gtk_event::*;

use flo_ui::*;
use flo_stream::*;

use futures::stream::{BoxStream};
use std::sync::*;

///
/// Represents a user interface running on Gtk
///
pub struct GtkUserInterface {
    /// The running thread used for this user interface
    thread: Arc<GtkThread>,

    /// The action input publisher
    input: Publisher<Vec<GtkAction>>
}

impl GtkUserInterface {
    ///
    /// Creates a new GTK user interface
    ///
    pub fn new() -> GtkUserInterface {
        // TODO: there should probably only be one thread, onto which we map the widget and window IDs used by this user interface
        // TODO: drop all of the widgets created by this user interface when this structure is dropped
        GtkUserInterface {
            thread: Arc::new(GtkThread::new()),
            input:  Publisher::new(100)
        }
    }
}

impl UserInterface<Vec<GtkAction>, GtkEvent, ()> for GtkUserInterface {
    type UpdateStream   = BoxStream<'static, Result<GtkEvent, ()>>;

    fn get_input_sink(&self) -> WeakPublisher<Vec<GtkAction>> {
        self.input.republish_weak()
    }

    fn get_updates(&self) -> Self::UpdateStream {
        Box::new(self.thread.get_event_stream())
    }
}
