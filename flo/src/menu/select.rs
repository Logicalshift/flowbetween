use super::controls;
use super::super::standard_tools::*;

use flo_ui::*;
use flo_canvas::*;
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
        let anything_selected   = tool_model.anything_selected.clone();
        let num_selected        = tool_model.num_elements_selected.clone();

        let ui              = 
            computed(move || {
                // Number of things selected
                let num_selected = num_selected.get();
                let num_selected = match num_selected {
                    0 => String::from("Nothing"),
                    1 => String::from("1 item"),
                    _ => format!("{} items", num_selected)
                };

                // Extra controls
                let selection_controls = if anything_selected.get() {
                    vec![
                        controls::divider(),

                        Control::label()
                            .with("Order:")
                            .with(TextAlign::Right)
                            .with(Font::Size(13.0))
                            .with(Bounds::next_horiz(48.0)),
                        Control::empty()
                            .with(Bounds::next_horiz(4.0)),
                        Control::container()
                            .with(Hint::Class("button-group".to_string()))
                            .with(ControlAttribute::Padding((0,2), (0,2)))
                            .with(Font::Size(9.0))
                            .with(Bounds::next_horiz(80.0))
                            .with(vec![
                                Control::button()
                                    .with(vec![Control::label().with("^^").with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with(Bounds::next_horiz(20.0)),
                                Control::button()
                                    .with(vec![Control::label().with("^").with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with(Bounds::next_horiz(20.0)),
                                Control::button()
                                    .with(vec![Control::label().with("v").with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with(Bounds::next_horiz(20.0)),
                                Control::button()
                                    .with(vec![Control::label().with("vv").with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with(Bounds::next_horiz(20.0))
                            ])
                    ]
                } else {
                    vec![]
                };

                // Build the control
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
                            .with(Font::Size(12.0))
                            .with(Bounds::next_horiz(56.0)),
                    ]
                    .into_iter()
                    .chain(selection_controls)
                    .collect::<Vec<_>>())
            });

        BindRef::from(ui)
    }
}

impl Controller for SelectMenuController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }
}
