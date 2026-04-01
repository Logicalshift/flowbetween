use super::logo_controller::*;

use flo_ui::*;
use flo_ui::Image;
use flo_ui_files::ui::*;
use flo_binding::*;
use flo_sqlite_storage::*;

use flo::style::*;
use flo::chooser::*;

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
    ui:         Binding<Control>,
    editor:     Arc<dyn Controller>,
    images:     Arc<ResourceManager<Image>>
}

impl FlowBetweenSession {
    ///
    /// Creates a new FlowBetween session
    ///
    pub fn new() -> FlowBetweenSession {
        let images = Arc::new(ResourceManager::new());

        // Some images for the root controller
        let flo = images.register(png_static(include_bytes!("../png/Flo-Orb-small.png")));
        images.assign_name(&flo, "flo");

        // Create the file chooser
        let file_chooser = FloChooser::new(Arc::new(sqlite_animation_loader()));
        let file_chooser = FileChooserController::new(file_chooser, FloLogoController::new());

        file_chooser.set_background(FILE_CHOOSER_BACKGROUND);

        // Create the session
        FlowBetweenSession {
            ui:         bind(Control::container()
                            .with(Bounds::fill_all())
                            .with(Appearance::Foreground(DEFAULT_TEXT))
                            .with(Appearance::Background(MAIN_BACKGROUND))
                            .with_controller(&serde_json::to_string(&SubController::Editor).unwrap())),
            editor:     Arc::new(file_chooser),
            images:     images
        }
    }

    /*
    fn create_inmemory_animation() -> SqliteAnimation {
        // Create a new animation
        let animation = SqliteAnimation::new_in_memory();

        let frame_length = animation.frame_length();

        // Add a single layer and an initial keyframe
        animation.perform_edits(vec![
            AnimationEdit::SetSize(1980.0, 1080.0),
            AnimationEdit::AddNewLayer(0),
            AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
            AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(frame_length*1)),
            AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(frame_length*2)),
            AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(frame_length*3)),
            AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(frame_length*4)),
            AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(frame_length*5))
        ]);

        animation
    }
    */
}

impl Controller for FlowBetweenSession {
    fn ui(&self) -> BindRef<Control> {
        BindRef::new(&self.ui)
    }

    fn get_subcontroller(&self, id: &str) -> Option<Arc<dyn Controller>> {
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
