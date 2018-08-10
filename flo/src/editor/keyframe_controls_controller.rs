use super::super::model::*;
use super::super::style::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

///
/// Provides the buttons for controlling the keyframes
/// 
pub struct KeyFrameControlsController {
    /// The UI for this controller
    ui: BindRef<Control>
}

impl KeyFrameControlsController {
    ///
    /// Creates a new keyframes controls controller
    /// 
    pub fn new<Anim: Animation+EditableAnimation>(model: &FloModel<Anim>) -> KeyFrameControlsController {
        let ui = Self::ui(model);

        KeyFrameControlsController {
            ui: ui
        }
    }

    ///
    /// Creates the UI for this controller
    /// 
    fn ui<Anim: Animation+EditableAnimation>(model: &FloModel<Anim>) -> BindRef<Control> {
        // Get the parts of the model we want to use

        // Create the UI
        let ui = computed(move || {
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
                        .with(Bounds::next_horiz(8.0)),
                    Control::container()
                        .with(Hint::Class("button-group".to_string()))
                        .with(vec![
                            Control::button()
                                .with(vec![Control::label().with("*").with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(Bounds::next_horiz(22.0)),
                            Control::button()
                                .with(vec![Control::label().with("*").with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(Bounds::next_horiz(22.0)),
                            Control::button()
                                .with(vec![Control::label().with("*").with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(Bounds::next_horiz(22.0)),
                            Control::button()
                                .with(vec![Control::label().with("*").with(TextAlign::Center).with(Bounds::fill_all())])
                                .with(Bounds::next_horiz(22.0)),
                        ])
                        .with(Bounds::next_horiz(22.0*4.0))

                ])
                .with(Bounds::fill_all())
        });

        // Turn into a bindref
        BindRef::from(ui)
    }
}

impl Controller for KeyFrameControlsController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }
}