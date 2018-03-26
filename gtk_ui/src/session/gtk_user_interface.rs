use super::super::gtk_thread::*;
use super::super::gtk_action::*;
use super::super::gtk_event::*;

use flo_ui::*;

use futures::*;
use std::sync::*;

///
/// Represents a user interface running on Gtk
/// 
pub struct GtkUserInterface {
    /// The running thread used for this user interface
    thread: Arc<GtkThread>
}

impl GtkUserInterface {
    ///
    /// Creates a new GTK user interface
    /// 
    pub fn new() -> GtkUserInterface {
        // TODO: there should probably only be one thread, onto which we map the widget and window IDs used by this user interface
        // TODO: drop all of the widgets created by this user interface when this structure is dropped
        GtkUserInterface {
            thread: Arc::new(GtkThread::new())
        }
    }    
}

impl UserInterface<Vec<GtkAction>, GtkEvent, ()> for GtkUserInterface {
    type EventSink      = Box<Sink<SinkItem=Vec<GtkAction>, SinkError=()>>;
    type UpdateStream   = Box<Stream<Item=GtkEvent, Error=()>>;

    fn get_input_sink(&self) -> Self::EventSink {
        unimplemented!()
    }

    fn get_updates(&self) -> Self::UpdateStream {
        Box::new(self.thread.get_event_stream())
    }
}
