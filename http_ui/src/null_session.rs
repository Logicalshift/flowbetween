use std::sync::*;

use super::session::*;
use super::session_state::*;

use ui::*;
use binding::*;

///
/// An empty session type that can be used for testing in the absense of an actual implementation
///
pub struct NullSession {
    ui:         BindRef<Control>
}

impl NullSession {
    pub fn new() -> NullSession {
        NullSession {
            ui: BindRef::from(computed(|| {
                Control::container()
                    .with(vec![Control::label().with("Hello, World")])
            }))
        }
    }
}

impl Session for NullSession {
    /// Creates a new session
    fn start_new(_state: Arc<SessionState>) -> Self {
        NullSession::new()
    }
}

impl Controller for NullSession {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn get_subcontroller(&self, _id: &str) -> Option<Arc<Controller>> {
        None
    }
}
