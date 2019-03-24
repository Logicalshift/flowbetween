use super::controls;
use super::super::standard_tools::*;

use flo_ui::*;
use flo_binding::*;

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
    pub fn new(tool_model: &SelectToolModel) -> SelectMenuController {
        let ui = Self::ui(tool_model);

        SelectMenuController {
            ui: ui
        }
    }

    ///
    /// Creates the UI for the select menu controller
    /// 
    fn ui(tool_model: &SelectToolModel) -> BindRef<Control> {
        let num_selected    = tool_model.num_elements_selected.clone();

        let ui              = 
            computed(move || {
                let num_selected = num_selected.get();
                let num_selected = match num_selected {
                    0 => String::from("Nothing"),
                    1 => String::from("1 item"),
                    _ => format!("{} items", num_selected)
                };

                Control::container()
                    .with(Bounds::fill_all())
                    .with(ControlAttribute::Padding((0, 3), (0, 3)))
                    .with(vec![
                        controls::divider(),

                        Control::label()
                            .with("Select:")
                            .with(FontWeight::Light)
                            .with(TextAlign::Right)
                            .with(Font::Size(14.0))
                            .with(Bounds::next_horiz(48.0)),

                        Control::label()
                            .with(num_selected)
                            .with(FontWeight::Light)
                            .with(TextAlign::Left)
                            .with(Font::Size(13.0))
                            .with(Bounds::next_horiz(64.0)),

                        controls::divider()
                    ])
            });

        BindRef::from(ui)
    }
}

impl Controller for SelectMenuController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }
}
