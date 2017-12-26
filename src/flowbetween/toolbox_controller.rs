use super::style::*;

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
    ///
    /// Creates a new toolbox controller
    /// 
    pub fn new() -> ToolboxController {
        // Create the viewmodel
        let viewmodel = Arc::new(DynamicViewModel::new());

        // There's a 'SelectedTool' key that describes the currently selected tool
        viewmodel.set_property("SelectedTool", PropertyValue::String("Pencil".to_string()));

        // Some images for the root controller
        let images  = Arc::new(Self::create_images());

        // Set up the tools
        let ui = bind(Control::container()
            .with(Bounds::fill_all())
            .with(ControlAttribute::Background(TOOLS_BACKGROUND))
            .with(vec![
                Self::make_tool("Select",   &viewmodel, images.get_named_resource("select")), 
                Self::make_tool("Adjust",   &viewmodel, images.get_named_resource("adjust")),
                Self::make_tool("Pan",      &viewmodel, images.get_named_resource("pan")),
                Self::make_separator(),
                Self::make_tool("Pencil",   &viewmodel, images.get_named_resource("pencil")), 
                Self::make_tool("Ink",      &viewmodel, images.get_named_resource("ink"))
            ]));

        ToolboxController {
            view_model: viewmodel,
            ui:         ui,
            images:     images
        }
    }

    ///
    /// Creates the image resources for this controller 
    ///
    fn create_images() -> ResourceManager<Image> {
        let images  = ResourceManager::new();

        // Load the tool images
        let select  = images.register(svg_static(include_bytes!("../../static_files/svg/tools/select.svg")));
        let adjust  = images.register(svg_static(include_bytes!("../../static_files/svg/tools/adjust.svg")));
        let pan     = images.register(svg_static(include_bytes!("../../static_files/svg/tools/pan.svg")));

        let pencil  = images.register(svg_static(include_bytes!("../../static_files/svg/tools/pencil.svg")));
        let ink     = images.register(svg_static(include_bytes!("../../static_files/svg/tools/ink.svg")));

        // Assign names to them
        images.assign_name(&select, "select");
        images.assign_name(&adjust, "adjust");
        images.assign_name(&pan, "pan");

        images.assign_name(&pencil, "pencil");
        images.assign_name(&ink, "ink");

        images
    }

    ///
    /// Creates a separator between controls
    /// 
    fn make_separator() -> Control {
        Control::empty()
            .with(Bounds::next_vert(12.0))
    }

    ///
    /// Creates a new tool control
    ///
    fn make_tool(name: &str, viewmodel: &DynamicViewModel, image: Option<Resource<Image>>) -> Control {
        use ui::ControlAttribute::*;
        use ui::ActionTrigger::*;

        // Decide if this is the selected tool
        let selected_tool   = viewmodel.get_property("SelectedTool");

        // The tool has a '-selected' binding that we use to cause it to highlight
        let compare_name            = String::from(name);
        let selected_property_name  = format!("{}-selected", name);

        // When the selected tool is set to the name of this tool, the selected property should be set to true
        viewmodel.set_computed(&selected_property_name, move || {
            let selected_tool = selected_tool.get().string().unwrap_or(String::from(""));
            PropertyValue::Bool(selected_tool == compare_name)
        });

        // The control is just a button
        Control::button()
            .with((Click, name))
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

    fn action(&self, action_id: &str, _action_parameter: &ActionParameter) {
        self.view_model.set_property("SelectedTool", PropertyValue::String(String::from(action_id)));
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(self.images.clone())
    }
}
