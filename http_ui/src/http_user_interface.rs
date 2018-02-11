use super::event::*;
use super::update::*;

use ui::*;
use ui::session::*;
use futures::*;
use futures::stream;

use std::sync::*;

///
/// Converts a core user interface into a HTTP user interface
/// 
pub struct HttpUserInterface<CoreUi> {
    core_ui: Arc<CoreUi>
}

impl<CoreUi: CoreUserInterface> HttpUserInterface<CoreUi> {
    ///
    /// Creates a new HTTP UI that will translate requests for the specified core UI
    /// 
    pub fn new(ui: Arc<CoreUi>) -> HttpUserInterface<CoreUi> {
        HttpUserInterface {
            core_ui: ui
        }
    }
}

impl<CoreUi: CoreUserInterface> UserInterface<Event, Vec<Update>, ()> for HttpUserInterface<CoreUi> {
    type EventSink = Box<Sink<SinkItem=Event, SinkError=()>>;
    type UpdateStream = Box<Stream<Item=Vec<Update>, Error=()>>;

    fn get_input_sink(&self) -> Self::EventSink {
        unimplemented!()
    }

    fn get_updates(&self) -> Self::UpdateStream {
        unimplemented!()
    }
}
