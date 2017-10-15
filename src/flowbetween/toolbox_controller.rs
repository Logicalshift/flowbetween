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
                Self::make_tool("Select",   &viewmodel), 
                Self::make_tool("Pan",      &viewmodel),
                Self::make_tool("Pencil",   &viewmodel), 
                Self::make_tool("Ink",      &viewmodel)
            ]));

        ToolboxController {
            view_model: viewmodel,
            ui:         ui
        }
    }

    ///
    /// Creates a new tool control
    ///
    fn make_tool(name: &str, viewmodel: &ToolboxViewModel) -> Control {
        use ui::ControlAttribute::*;
        use ui::ActionTrigger::*;

        // The tool has a '-selected' binding that we use to cause it to highlight
        let selected_property_name = format!("{}-selected", name);
        viewmodel.set_property(&selected_property_name, PropertyValue::Bool(false));

        // The control is just a button
        Control::button()
            .with(Action(Click, String::from(name)))
            .with(Selected(Property::Bind(selected_property_name)))
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
