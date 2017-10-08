use ui::*;

///
/// The toolbox controller allows the user to pick which tool they
/// are using to edit the canvas
///
pub struct ToolboxController {
    ui: Binding<Control>
}

impl ToolboxController {
    pub fn new() -> ToolboxController {
        let ui = bind(Control::empty());

        ToolboxController {
            ui: ui
        }
    }
}

impl Controller for ToolboxController {
    fn ui(&self) -> Box<Bound<Control>> {
        Box::new(self.ui.clone())
    }
}
