use super::super::model::*;
use super::super::style::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;

///
/// Provides the buttons for controlling the keyframes
/// 
pub struct KeyFrameControlsController {
    /// The UI for this controller
    ui: BindRef<Control>,

    /// The images for this controller
    images: Arc<ResourceManager<Image>>
}

impl KeyFrameControlsController {
    ///
    /// Creates a new keyframes controls controller
    /// 
    pub fn new<Anim: Animation+EditableAnimation>(model: &FloModel<Anim>) -> KeyFrameControlsController {
        let images  = Arc::new(Self::images());
        let ui      = Self::ui(model, Arc::clone(&images));

        KeyFrameControlsController {
            ui:     ui,
            images: images
        }
    }

    ///
    /// Creates the UI for this controller
    /// 
    fn ui<Anim: Animation+EditableAnimation>(model: &FloModel<Anim>, images: Arc<ResourceManager<Image>>) -> BindRef<Control> {
        // Fetch the icon images
        let new_key_frame       = images.get_named_resource("new_key_frame").unwrap();
        let new_on_paint        = images.get_named_resource("new_on_paint").unwrap();
        let next_key_frame      = images.get_named_resource("next_key_frame").unwrap();
        let onion_skins         = images.get_named_resource("onion_skins").unwrap();
        let previous_key_frame  = images.get_named_resource("previous_key_frame").unwrap();

        // Get the parts of the model we want to use

        // Create the UI
        let ui = computed(move || {
            let new_key_frame       = new_key_frame.clone();
            let new_on_paint        = new_on_paint.clone();
            let next_key_frame      = next_key_frame.clone();
            let onion_skins         = onion_skins.clone();
            let previous_key_frame  = previous_key_frame.clone();

            Control::container()
                .with(vec![
                    Control::empty()
                        .with(Appearance::Background(TIMESCALE_LAYERS))
                        .with(Bounds::next_horiz(1.0)),
                    Control::empty()
                        .with(Bounds::next_horiz(3.0)),
                    Control::label()
                        .with("Keyframes:")
                        .with(TextAlign::Right)
                        .with(Bounds::stretch_horiz(1.0)),
                    Control::empty()
                        .with(Bounds::next_horiz(10.0)),
                    Control::container()
                        .with(Hint::Class("button-group".to_string()))
                        .with(vec![
                            Control::button()
                                .with(vec![Control::label().with(previous_key_frame).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((5, 2), (2, 2)))
                                .with(Bounds::next_horiz(22.0)),
                            Control::button()
                                .with(vec![Control::label().with(new_key_frame).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((2, 2), (2, 2)))
                                .with(Bounds::next_horiz(22.0)),
                            Control::button()
                                .with(vec![Control::label().with(onion_skins).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((2, 2), (2, 2)))
                                .with(Bounds::next_horiz(22.0)),
                            Control::button()
                                .with(vec![Control::label().with(new_on_paint).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((2, 2), (2, 2)))
                                .with(Bounds::next_horiz(22.0)),
                            Control::button()
                                .with(vec![Control::label().with(next_key_frame).with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(ControlAttribute::Padding((2, 2), (5, 2)))
                                .with(Bounds::next_horiz(22.0)),
                        ])
                        .with(Bounds::next_horiz(22.0*5.0))

                ])
                .with(Bounds::fill_all())
        });

        // Turn into a bindref
        BindRef::from(ui)
    }

    ///
    /// Creates the image resource manager for this controller
    /// 
    fn images() -> ResourceManager<Image> {
        let images              = ResourceManager::new();

        let new_key_frame       = images.register(svg_static(include_bytes!("../../svg/keyframes/new_key_frame.svg")));
        let new_on_paint        = images.register(svg_static(include_bytes!("../../svg/keyframes/new_on_paint.svg")));
        let next_key_frame      = images.register(svg_static(include_bytes!("../../svg/keyframes/next_key_frame.svg")));
        let onion_skins         = images.register(svg_static(include_bytes!("../../svg/keyframes/onion_skins.svg")));
        let previous_key_frame  = images.register(svg_static(include_bytes!("../../svg/keyframes/previous_key_frame.svg")));

        images.assign_name(&new_key_frame,      "new_key_frame");
        images.assign_name(&new_on_paint,       "new_on_paint");
        images.assign_name(&next_key_frame,     "next_key_frame");
        images.assign_name(&onion_skins,        "onion_skins");
        images.assign_name(&previous_key_frame, "previous_key_frame");

        images
    }
}

impl Controller for KeyFrameControlsController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(Arc::clone(&self.images))
    }
}