use flo_ui::*;
use flo_stream::*;
use flo_cocoa_pipe::*;

use std::sync::*;

///
/// Basic Cocoa user interface
///
pub struct CocoaUserInterface {
    actions:    Publisher<Vec<AppAction>>,
    events:     Mutex<Publisher<Vec<AppEvent>>>
}

impl CocoaUserInterface {
    ///
    /// Creates a new Cocoa user interface
    ///
    pub fn new(actions: Publisher<Vec<AppAction>>, events: Publisher<Vec<AppEvent>>) -> CocoaUserInterface {
        CocoaUserInterface {
            actions:    actions,
            events:     Mutex::new(events)
        }
    }
}

impl UserInterface<Vec<AppAction>, Vec<AppEvent>, ()> for CocoaUserInterface {
    type EventSink      = Publisher<Vec<AppAction>>;
    type UpdateStream   = Subscriber<Vec<AppEvent>>;

    fn get_input_sink(&self) -> Self::EventSink {
        self.actions.republish()
    }

    fn get_updates(&self) -> Self::UpdateStream {
        self.events.lock().unwrap().subscribe()
    }
}
