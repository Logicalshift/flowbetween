use super::state::*;

///
/// Core UI session structures
/// 
pub struct UiSessionCore {
    /// The state of the UI at the last update
    state: UiSessionState
}

impl UiSessionCore {
    pub fn new() -> UiSessionCore {
        UiSessionCore {
            state: UiSessionState::new()
        }
    }
}
