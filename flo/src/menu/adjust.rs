use super::controls;

use flo_ui::*;
use flo_binding::*;

///
/// The menu controller for the adjust tool
///
pub struct AdjustMenuController {
    ui: BindRef<Control>
}

impl AdjustMenuController {
    ///
    /// Creates a new adjust menu controller
    ///
    pub fn new() -> AdjustMenuController {
        let ui = Self::ui();

        AdjustMenuController {
            ui: ui
        }
    }

    ///
    /// Creates the UI for the adjust menu controller
    ///
    fn ui() -> BindRef<Control> {
        let ui = bind(Control::container()
                    .with(Bounds::fill_all())
                    .with(ControlAttribute::Padding((0, 3), (0, 3)))
                    .with(vec![
                        controls::divider(),

                        Control::label()
                            .with("Adjust:")
                            .with(FontWeight::Light)
                            .with(TextAlign::Right)
                            .with(Font::Size(14.0))
                            .with(Bounds::next_horiz(48.0))
                    ])
            );

        BindRef::from(ui)
    }
}

impl Controller for AdjustMenuController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }
}
