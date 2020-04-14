use super::keyframe_controls_controller::*;
use super::super::model::*;
use super::super::style::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;

///
/// The control bar provides a home for the basic animation editing and playback controls
///
pub struct ControlBarController<Anim: 'static+Animation+EditableAnimation> {
    /// The UI for this control bar
    ui: BindRef<Control>,

    /// The keyframe controls controller
    keyframe_controls: Arc<KeyFrameControlsController<Anim>>,

    /// The images for this controller
    images: Arc<ResourceManager<Image>>
}

impl<Anim: 'static+Animation+EditableAnimation> ControlBarController<Anim> {
    ///
    /// Creates a new control bar controller
    ///
    pub fn new(model: &FloModel<Anim>) -> ControlBarController<Anim> {
        // Create the UI
        let images              = Arc::new(Self::images());
        let ui                  = Self::ui(Arc::clone(&images));

        // Create the subcontrollers
        let keyframe_controls   = KeyFrameControlsController::new(model);
        let keyframe_controls   = Arc::new(keyframe_controls);

        // Build the controller itself
        ControlBarController {
            ui:                 ui,
            keyframe_controls:  keyframe_controls,
            images:             images
        }
    }

    ///
    /// Creates the image resource manager for this controller
    ///
    fn images() -> ResourceManager<Image> {
        let images              = ResourceManager::new();

        let frame_controls      = images.register(svg_static(include_bytes!("../../svg/keyframes/frame_controls.svg")));

        images.assign_name(&frame_controls,     "frame_controls");

        images
    }

    ///
    /// Creates the UI for this controller
    ///
    fn ui(images: Arc<ResourceManager<Image>>) -> BindRef<Control> {
        let frame_controls = images.get_named_resource("frame_controls");

        // Create the UI itself
        let ui = Control::container()
            .with(Bounds::fill_all())
            .with(ControlAttribute::Padding((0, 2), (0, 2)))
            .with(vec![
                Control::empty()
                    .with(Bounds::next_horiz(6.0)),

                Control::container()
                    .with(frame_controls.clone())
                    .with(vec![
                        Control::empty()
                            .with(ControlAttribute::Padding((9, 4), (4, 4)))
                            .with(Bounds::next_horiz(22.0)),
                        Control::empty()
                            .with(ControlAttribute::Padding((9, 4), (4, 4)))
                            .with(Bounds::next_horiz(22.0)),
                        Control::empty()
                            .with(ControlAttribute::Padding((4, 4), (4, 4)))
                            .with(Bounds::next_horiz(22.0)),
                        Control::empty()
                            .with(ControlAttribute::Padding((4, 4), (4, 4)))
                            .with(Bounds::next_horiz(22.0)),
                        Control::empty()
                            .with(ControlAttribute::Padding((4, 4), (4, 4)))
                            .with(Bounds::next_horiz(22.0)),
                        Control::empty()
                            .with(ControlAttribute::Padding((4, 4), (4, 4)))
                            .with(Bounds::next_horiz(22.0)),

                        Control::empty()
                            .with(Bounds::next_horiz(4.0)),
                        Control::label()
                            .with("F 107999")
                            .with(TextAlign::Left)
                            .with(Font::Size(11.0))
                            .with(Font::Weight(FontWeight::Normal))
                            .with(ControlAttribute::Padding((4, 4), (9, 4)))
                            .with(Bounds::next_horiz(76.0))
                    ])
                    .with(Bounds::next_horiz(22.0*6.0+80.0)),

                Control::empty()
                    .with(Bounds::next_horiz(3.0)),
                Control::empty()
                    .with(Appearance::Background(TIMESCALE_LAYERS))
                    .with(Bounds::next_horiz(1.0)),

                Control::empty()
                    .with(Bounds::stretch_horiz(1.0)),
                Control::container()
                    .with_controller("KeyFrameControls")
                    .with(Bounds::next_horiz(188.0)),
                Control::empty()
                    .with(Bounds::next_horiz(32.0))
            ]);

        // Create the binding
        let ui = bind(ui);
        BindRef::from(ui)
    }
}

impl<Anim: 'static+Animation+EditableAnimation> Controller for ControlBarController<Anim> {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn get_subcontroller(&self, id: &str) -> Option<Arc<dyn Controller>> {
        match id {
            "KeyFrameControls"  => Some(self.keyframe_controls.clone()),

            _                   => None
        }
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(Arc::clone(&self.images))
    }
}
