use super::editor_controller::*;

use ui::*;
use ui::Image;
use binding::*;
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
    view_model: Arc<NullViewModel>,
    ui:         Binding<Control>,
    editor:     Arc<Controller>,
    images:     Arc<ResourceManager<Image>>
}

impl FlowBetweenSession {
    pub fn new() -> FlowBetweenSession {
        let images = Arc::new(ResourceManager::new());

        // Some images for the root controller
        let flo = images.register(png_static(include_bytes!("../../static_files/png/Flo-Orb-small.png")));
        images.assign_name(&flo, "flo");

        // Create the session
        FlowBetweenSession {
            view_model: Arc::new(NullViewModel::new()),
            ui:         bind(Control::container()
                            .with(Bounds::fill_all())
                            .with_controller(&serde_json::to_string(&SubController::Editor).unwrap())),
            editor:     Arc::new(EditorController::new(())),
            images:     images
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
    fn ui(&self) -> Arc<Bound<Control>> {
        Arc::new(self.ui.clone())
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

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(self.images.clone())
    }
}
