use flo_ui::*;
use flo_stream::*;
use flo_cocoa_pipe::*;

use std::sync::*;

///
/// Basic Cocoa user interface
///
pub struct CocoaUserInterface {
    actions:    Publisher<AppAction>,
    events:     Mutex<Publisher<AppEvent>>
}

impl CocoaUserInterface {
    ///
    /// Creates a new Cocoa user interface
    ///
    pub fn new(actions: Publisher<AppAction>, events: Publisher<AppEvent>) -> CocoaUserInterface {
        CocoaUserInterface {
            actions:    actions,
            events:     Mutex::new(events)
        }
    }
}

impl UserInterface<AppAction, AppEvent, ()> for CocoaUserInterface {
    type EventSink = Publisher<AppAction>;
    type UpdateStream = Subscriber<AppEvent>;

    fn get_input_sink(&self) -> Self::EventSink {
        self.actions.republish()
    }

    fn get_updates(&self) -> Self::UpdateStream {
        self.events.lock().unwrap().subscribe()
    }
}
