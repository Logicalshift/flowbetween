use ui::*;
use binding::*;

use std::sync::*;

///
/// The toolbox controller allows the user to pick which tool they
/// are using to edit the canvas
///
pub struct ToolboxController {
    view_model: Arc<DynamicViewModel>,
    ui:         Binding<Control>
}

impl ToolboxController {
    pub fn new() -> ToolboxController {
        // Create the viewmodel
        let viewmodel = Arc::new(DynamicViewModel::new());

        // There's a 'SelectedTool' key that describes the currently selected tool
        viewmodel.set_property("SelectedTool", PropertyValue::String("Pencil".to_string()));

        // Set up the tools
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
    fn make_tool(name: &str, viewmodel: &DynamicViewModel) -> Control {
        use ui::ControlAttribute::*;
        use ui::ActionTrigger::*;

        // Decide if this is the selected tool
        let selected_tool   = viewmodel.get_property("SelectedTool");

        // The tool has a '-selected' binding that we use to cause it to highlight
        let compare_name            = String::from(name);
        let selected_property_name  = format!("{}-selected", name);
        viewmodel.set_computed(&selected_property_name, move || {
            let selected_tool = selected_tool.get().string().unwrap_or(String::from(""));
            PropertyValue::Bool(selected_tool == compare_name)
        });

        // The control is just a button
        Control::button()
            .with(Action(Click, String::from(name)))
            .with(Selected(Property::Bind(selected_property_name)))
            .with(Bounds::next_vert(48.0))
    }
}

impl Controller for ToolboxController {
    fn ui(&self) -> Arc<Bound<Control>> {
        Arc::new(self.ui.clone())
    }

    fn action(&self, action_id: &str) {
        self.view_model.set_property("SelectedTool", PropertyValue::String(String::from(action_id)));
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }
}
