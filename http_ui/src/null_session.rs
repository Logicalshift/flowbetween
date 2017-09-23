use std::sync::*;

use super::session::*;
use super::session_state::*;

///
/// An empty session type that can be used for testing in the absense of an actual implementation
///
pub struct NullSession {
}

impl NullSession {
    pub fn new() -> NullSession {
        NullSession {}
    }
}

impl Session for NullSession {
    /// Creates a new session
    fn start_new(state: Arc<SessionState>) -> Self {
        NullSession::new()
    }
}
