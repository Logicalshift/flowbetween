use ui::*;

use uuid::*;
use std::sync::*;

///
/// The session state object represents the stored state of a particular session
///
pub struct SessionState {
    /// A string identifying this session
    session_id: String,

    /// The UI tree for this session
    ui_tree: Mutex<Box<Bound<Control>>>
}

impl SessionState {
    ///
    /// Creates a new session state
    ///
    pub fn new() -> SessionState {
        let session_id = Uuid::new_v4().simple().to_string();

        SessionState { 
            session_id: session_id,
            ui_tree:    Mutex::new(Box::new(bind(Control::container())))
        }
    }

    ///
    /// Retrieves the ID of this session
    ///
    pub fn id(&self) -> String {
        self.session_id.clone()
    }

    ///
    /// Replaces the UI tree in this session
    ///
    pub fn set_ui_tree<TBinding: 'static+Bound<Control>>(&self, new_tree: TBinding) {
        *self.ui_tree.lock().unwrap() = Box::new(new_tree);
    }

    ///
    /// Retrieves the current state of the UI for this session
    ///
    pub fn entire_ui_tree(&self) -> Control {
        self.ui_tree.lock().unwrap().get()
    }
}