use ui::*;
use ui::Image;
use binding::*;
use http_ui::*;
use animation::*;
use animation::inmemory::*;

use flo::*;
use flo::style::*;

use std::sync::*;
use std::time::Duration;
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
    ui:         Binding<Control>,
    editor:     Arc<Controller>,
    images:     Arc<ResourceManager<Image>>
}

impl FlowBetweenSession {
    ///
    /// Creates a new FlowBetween session
    ///
    pub fn new() -> FlowBetweenSession {
        let images = Arc::new(ResourceManager::new());

        // Create a new animation
        let animation = Self::create_inmemory_animation();

        // Some images for the root controller
        let flo = images.register(png_static(include_bytes!("../static_files/png/Flo-Orb-small.png")));
        images.assign_name(&flo, "flo");

        // Create the session
        FlowBetweenSession {
            ui:         bind(Control::container()
                            .with(Bounds::fill_all())
                            .with(Appearance::Foreground(DEFAULT_TEXT))
                            .with(Appearance::Background(MAIN_BACKGROUND))
                            .with_controller(&serde_json::to_string(&SubController::Editor).unwrap())),
            editor:     Arc::new(EditorController::new(animation)),
            images:     images
        }
    }

    fn create_inmemory_animation() -> InMemoryAnimation {
        // Create a new animation
        let animation = InMemoryAnimation::new();

        // Add a single layer and an initial keyframe
        animation.perform_edits(vec![
            AnimationEdit::SetSize(1980.0, 1080.0),
            AnimationEdit::AddNewLayer(0)
        ]);

        {
            // TODO: this is not using the edit log method...
            // Add a single layer and an initial keyframe
            let layers = open_edit::<AnimationLayers>(&animation).unwrap();
            let layer = layers.layers()
                .filter(|layer| layer.id() == 0)
                .nth(0)
                .unwrap();

            let mut keyframes: Editor<KeyFrameLayer> = layer.edit().unwrap();
            keyframes.add_key_frame(Duration::from_millis(0));
        }
        
        animation
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
    fn ui(&self) -> BindRef<Control> {
        BindRef::new(&self.ui)
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

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(self.images.clone())
    }
}
