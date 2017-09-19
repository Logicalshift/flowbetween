use std::sync::*;

use super::session_state::*;

///
/// Implementations of the session trait supply the UI and event
/// handling information for a single session.
///
pub trait Session : Send {
    /// Creates a new session
    fn start_new(state: Arc<SessionState>) -> Self;
}

