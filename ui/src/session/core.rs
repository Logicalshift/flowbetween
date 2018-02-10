use super::state::*;
use super::event::*;
use super::super::controller::*;

///
/// Core UI session structures
/// 
pub struct UiSessionCore {
    /// The state of the UI at the last update
    state: UiSessionState
}

impl UiSessionCore {
    ///
    /// Creates a new UI core
    /// 
    pub fn new() -> UiSessionCore {
        UiSessionCore {
            state: UiSessionState::new()
        }
    }

    ///
    /// Dispatches an event to the specified controller
    ///  
    pub fn dispatch_event(&mut self, _event: UiEvent, _controller: &Controller) {

    }
}
