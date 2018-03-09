use super::controls;

use ui::*;
use binding::*;

///
/// The menu controller for the selection tool
/// 
pub struct SelectMenuController {
    ui: BindRef<Control>
}

impl SelectMenuController {
    ///
    /// Creates a new select menu controller
    /// 
    pub fn new() -> SelectMenuController {
        let ui = Self::ui();

        SelectMenuController {
            ui: ui
        }
    }

    ///
    /// Creates the UI for the select menu controller
    /// 
    fn ui() -> BindRef<Control> {
        let ui = bind(Control::container()
                    .with(Bounds::fill_all())
                    .with(ControlAttribute::Padding((0, 3), (0, 3)))
                    .with(vec![
                        controls::divider(),

                        Control::label()
                            .with("Select:")
                            .with(FontWeight::Light)
                            .with(TextAlign::Right)
                            .with(Font::Size(14.0))
                            .with(Bounds::next_horiz(48.0))
                    ])
            );

        BindRef::from(ui)
    }
}

impl Controller for SelectMenuController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }
}
