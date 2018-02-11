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
    /// The core UI is the non-platform specific implementation of the user interface
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

    ///
    /// Retrieves the underlying non-platform specific UI object
    /// 
    pub fn core(&self) -> Arc<CoreUi> {
        Arc::clone(&self.core_ui)
    }

    ///
    /// Converts an event from the HTTP side of things into a UI event
    /// 
    fn http_event_to_core_event(http_event: Event) -> UiEvent {
        use Event::*;

        match http_event {
            NewSession  => UiEvent::Tick,
            UiRefresh   => UiEvent::Tick,
            Tick        => UiEvent::Tick,

            Action(controller_path, action_name, action_parameter) => UiEvent::Action(controller_path, action_name, action_parameter)
        }
    }
}

impl<CoreUi: CoreUserInterface> UserInterface<Event, Vec<Update>, ()> for HttpUserInterface<CoreUi> {
    type EventSink = Box<Sink<SinkItem=Event, SinkError=()>>;
    type UpdateStream = Box<Stream<Item=Vec<Update>, Error=()>>;

    fn get_input_sink(&self) -> Self::EventSink {
        let core_sink   = self.core_ui.get_input_sink();
        let mapped_sink = core_sink.with_flat_map(|http_event| {
            let core_event = Self::http_event_to_core_event(http_event);
            stream::once(Ok(core_event))
        });

        Box::new(mapped_sink)
    }

    fn get_updates(&self) -> Self::UpdateStream {
        unimplemented!()
    }
}
