use flo_ui::*;
use flo_stream::*;
use flo_cocoa_pipe::*;

use std::sync::*;

use futures::*;
use futures::stream::{BoxStream};

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
    type UpdateStream   = BoxStream<'static, Result<Vec<AppEvent>, ()>>;

    fn get_input_sink(&self) -> WeakPublisher<Vec<AppAction>> {
        self.actions.republish_weak()
    }

    fn get_updates(&self) -> Self::UpdateStream {
        self.events.lock().unwrap().subscribe().map(|msg| Ok(msg)).boxed()
    }
}
