use super::toolbox_viewmodel::*;

use ui::*;

use std::sync::*;

///
/// The toolbox controller allows the user to pick which tool they
/// are using to edit the canvas
///
pub struct ToolboxController {
    view_model: Arc<ToolboxViewModel>,
    ui:         Binding<Control>
}

impl ToolboxController {
    pub fn new() -> ToolboxController {
        let viewmodel = Arc::new(ToolboxViewModel::new());

        let ui = bind(Control::container()
            .with(Bounds::fill_all())
            .with(vec![
                Self::make_tool(), 
                Self::make_tool(),
                Self::make_tool(), 
                Self::make_tool()
            ]));

        ToolboxController {
            view_model: viewmodel,
            ui:         ui
        }
    }

    ///
    /// Creates a new tool control
    ///
    fn make_tool() -> Control {
        use ui::ControlAttribute::*;
        use ui::ActionTrigger::*;

        Control::button()
            .with(Action(Click, String::from("ToolClick")))
            .with(Bounds::next_vert(48.0))
    }
}

impl Controller for ToolboxController {
    fn ui(&self) -> Box<Bound<Control>> {
        Box::new(self.ui.clone())
    }

    fn action(&self, action_id: &str) {
        println!("Toolbox action is {}", action_id);
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }
}
