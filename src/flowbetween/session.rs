use super::editor_controller::*;

use ui::*;
use http_ui::*;

use std::sync::*;
use serde_json;

///
/// Possible subcontrollers of the main flowbetween controller
///
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
enum SubController {
    Editor
}

///
/// The main flowbetween session object
///
pub struct FlowBetweenSession {
    ui:             Binding<Control>,
    editor: Arc<EditorController>
}

impl FlowBetweenSession {
    pub fn new() -> FlowBetweenSession {
        FlowBetweenSession {
            ui: bind(Control::container()
                    .with(Bounds::fill_all())
                    .with_controller(&serde_json::to_string(&SubController::Editor).unwrap())),
            editor: Arc::new(EditorController::new())
        }
    }
}

impl Session for FlowBetweenSession {
    /// Creates a new session
    fn start_new(_state: Arc<SessionState>) -> Self {
        let session = FlowBetweenSession::new();

        session
    }
}

impl Controller for FlowBetweenSession {
    fn ui(&self) -> Box<Bound<Control>> {
        Box::new(self.ui.clone())
    }

    fn get_subcontroller(&self, id: &str) -> Option<Arc<Controller>> {
        let id = serde_json::from_str(id);

        if let Ok(id) = id {
            match id {
                SubController::Editor => Some(self.editor.clone())
            }
        } else {
            None
        }
    }
}
