use ui::*;
use binding::*;

use std::sync::*;

///
/// The toolbox controller allows the user to pick which tool they
/// are using to edit the canvas
///
pub struct ToolboxController {
    view_model: Arc<DynamicViewModel>,
    ui:         Binding<Control>,
    images:     Arc<ResourceManager<Image>>
}

impl ToolboxController {
    pub fn new() -> ToolboxController {
        // Create the viewmodel
        let viewmodel = Arc::new(DynamicViewModel::new());

        // There's a 'SelectedTool' key that describes the currently selected tool
        viewmodel.set_property("SelectedTool", PropertyValue::String("Pencil".to_string()));

        // Some images for the root controller
        let images  = Arc::new(ResourceManager::new());
        let flo     = images.register(png_static(include_bytes!("../../static_files/png/Flo-Orb-small.png")));
        images.assign_name(&flo, "flo");

        // Set up the tools
        let ui = bind(Control::container()
            .with(Bounds::fill_all())
            .with(vec![
                Self::make_tool("Select",   &viewmodel, flo.clone()), 
                Self::make_tool("Pan",      &viewmodel, flo.clone()),
                Self::make_tool("Pencil",   &viewmodel, flo.clone()), 
                Self::make_tool("Ink",      &viewmodel, flo.clone())
            ]));

        ToolboxController {
            view_model: viewmodel,
            ui:         ui,
            images:     images
        }
    }

    ///
    /// Creates a new tool control
    ///
    fn make_tool(name: &str, viewmodel: &DynamicViewModel, image: Resource<Image>) -> Control {
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
            .with(vec![
                Control::empty()
                    .with(Bounds::fill_all())
                    .with(image)
            ])
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

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(self.images.clone())
    }
}
