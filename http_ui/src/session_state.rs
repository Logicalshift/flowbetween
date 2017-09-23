use ui::*;

use uuid::*;

///
/// The session state object represents the stored state of a particular session
///
pub struct SessionState {
    /// A string identifying this session
    session_id: String
}

impl SessionState {
    ///
    /// Creates a new session state
    ///
    pub fn new() -> SessionState {
        let session_id = Uuid::new_v4().simple().to_string();

        SessionState { session_id: session_id }
    }

    ///
    /// Retrieves the ID of this session
    ///
    pub fn id(&self) -> String {
        self.session_id.clone()
    }

    ///
    /// Retrieves the current state of the UI for this session
    ///
    pub fn entire_ui_tree(&self) -> Control {
        // TODO: this is just a placeholder
        let hello_world = Control::container()
            .with(vec![Control::label().with("Hello, World")]);

        hello_world.clone()
    }
}