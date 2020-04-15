use super::super::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;
use std::time::Duration;

///
/// The frame controls allows for choosing individual frames and playback
///
/// It differs from the similarly named keyframe controller in that not every frame contains
/// a keyframe
///
pub struct FrameControlsController<Anim: 'static+Animation+EditableAnimation> {
    /// The UI for this controller
    ui: BindRef<Control>,

    /// The images for this controller
    images: Arc<ResourceManager<Image>>,

    /// The view model for this controller
    view_model: Arc<DynamicViewModel>,

    /// The frame model
    frame: FrameModel,

    /// The timeline model
    timeline: TimelineModel<Anim>,

    /// The current frame binding
    current_time: Binding<Duration>,
}

impl<Anim: 'static+Animation+EditableAnimation> FrameControlsController<Anim> {
    ///
    /// Creates a new keyframes controls controller
    ///
    pub fn new(model: &FloModel<Anim>) -> FrameControlsController<Anim> {
        // Create the viewmodel
        let frame       = model.frame();
        let timeline    = model.timeline();
        let view_model  = Arc::new(DynamicViewModel::new());

        // Create the images and the UI
        let images          = Arc::new(Self::images());
        let ui              = Self::ui(Arc::clone(&images));

        FrameControlsController {
            ui:             ui,
            images:         images,
            view_model:     view_model,
            frame:          frame.clone(),
            timeline:       timeline.clone(),
            current_time:   timeline.current_time.clone(),
        }
    }

    ///
    /// Creates the UI for this controller
    ///
    fn ui(images: Arc<ResourceManager<Image>>) -> BindRef<Control> {
        BindRef::new(&Binding::new(Control::empty()))
    }

    ///
    /// Creates the image resource manager for this controller
    ///
    fn images() -> ResourceManager<Image> {
        ResourceManager::new()
    }
}

impl<Anim: 'static+Animation+EditableAnimation> Controller for FrameControlsController<Anim> {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(Arc::clone(&self.images))
    }
}
