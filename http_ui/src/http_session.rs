use super::http_user_interface::*;

use ui::*;
use ui::session::*;

///
/// Represents a session running on a HTTP connection 
/// 
pub struct HttpSession<CoreUi> {
    /// The core UI object
    http_ui: HttpUserInterface<CoreUi>,

    /// The event sink for the UI
    input: HttpEventSink,

    /// The stream of events for the session (or None if it has been reset or not started yet)
    updates: Option<HttpUpdateStream>
}

impl<CoreUi: 'static+CoreUserInterface> HttpSession<CoreUi> {
    ///
    /// Creates a new session from a HTTP user interface
    /// 
    pub fn new(http_ui: HttpUserInterface<CoreUi>) -> HttpSession<CoreUi> {
        let input = http_ui.get_input_sink();

        HttpSession {
            http_ui:    http_ui,
            input:      input,
            updates:    None
        }
    }

    ///
    /// Retrieves the input event sink for this session
    /// 
    pub fn input(&mut self) -> &mut HttpEventSink {
        &mut self.input
    }

    ///
    /// Retrieves the update stream for this session
    /// 
    pub fn updates(&mut self) -> &mut HttpUpdateStream {
        loop {
            if let Some(ref mut updates) = self.updates {
                // Return existing updates
                return updates;
            } else {
                // Start a new update stream
                self.updates = Some(self.http_ui.get_updates());
            }
        }
    }
}
