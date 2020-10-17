use super::super::tools::*;
use super::super::style::*;
use super::super::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;

pub const TOOL_CONTROL_SIZE: f32 = 32.0;

///
/// The toolbox controller allows the user to pick which tool they
/// are using to edit the canvas
///
pub struct ToolboxController<Anim: Animation> {
    view_model:         Arc<DynamicViewModel>,
    ui:                 BindRef<Control>,
    images:             Arc<ResourceManager<Image>>,
    anim_model:         FloModel<Anim>
}

impl<Anim: 'static+EditableAnimation+Animation> ToolboxController<Anim> {
    ///
    /// Creates a new toolbox controller
    ///
    pub fn new(anim_model: &FloModel<Anim>) -> ToolboxController<Anim> {
        // Create the viewmodel
        let viewmodel = Arc::new(DynamicViewModel::new());

        // There's a 'SelectedTool' key that describes the currently selected tool
        viewmodel.set_property("SelectedTool", PropertyValue::String("Ink".to_string()));

        // Update the viewmodel whenever the effective tool changes
        let effective_tool = anim_model.tools().effective_tool.clone();
        viewmodel.set_computed("EffectiveTool", move ||
            PropertyValue::String(effective_tool.get().map(|tool| tool.tool_name()).unwrap_or("".to_string())));

        // Make sure that the tool selected in this controller matches the one in the main view model
        anim_model.tools().choose_tool_with_name(&viewmodel.get_property("SelectedTool").get().string().unwrap_or("".to_string()));

        // Some images for the root controller
        let images  = Arc::new(Self::create_images(anim_model));

        // Set up the tools
        let ui = Self::create_ui(anim_model, Arc::clone(&viewmodel), Arc::clone(&images));

        ToolboxController {
            view_model:         viewmodel,
            ui:                 ui,
            anim_model:         anim_model.clone(),
            images:             images
        }
    }

    ///
    /// Creates the UI binding
    ///
    fn create_ui(anim_model: &FloModel<Anim>, viewmodel: Arc<DynamicViewModel>, images: Arc<ResourceManager<Image>>) -> BindRef<Control> {
        let tools                   = anim_model.tools();

        // Create a control binding for each toolset
        let selected_tool_set       = tools.selected_tool_set.clone();
        let selected_tool_for_set   = tools.selected_tool_for_set.clone();
        let tool_sets               = tools.tool_sets.clone();
        let binding_viewmodel       = viewmodel.clone();
        let binding_images          = images.clone();

        let tool_set_selector       = computed(move || {
            // Fetch the tool sets
            let tool_sets = tool_sets.get();

            // Create selected indicators for each toolset (TODO: should really be outside of the 'computed' block)
            for set in tool_sets.iter() {
                let selected_tool_set   = selected_tool_set.clone();
                let ToolSetId(id)       = set.id();

                binding_viewmodel.set_computed(&format!("toolset-selected-{}", id), move || {
                    PropertyValue::Bool(selected_tool_set.get() == Some(ToolSetId(id.clone())))
                });
            }

            // Create the tool sets themselves
            let tool_set_controls = tool_sets.iter()
                .flat_map(|tool_set| {
                    // Get the currently selected tool
                    let selected_tool = selected_tool_for_set.lock().unwrap()
                        .entry(tool_set.id())
                        .or_insert_with(|| bind(None))
                        .get()
                        .or_else(|| tool_set.tools().iter().nth(0).cloned())?;

                    // Fetch the image for the selected tool (this is what we use to represent the tool set, as it's the tool you get if you select this set)
                    let image_name              = format!("tool-{}", selected_tool.tool_name());
                    let image                   = binding_images.get_named_resource(&image_name);

                    let ToolSetId(toolset_id)   = tool_set.id();
                    let selected_property_name  = format!("toolset-selected-{}", toolset_id);

                    // Turn into a control
                    let control     =         Control::button()
                        /* .with((Click, name)) */
                        .with(State::Selected(Property::Bind(selected_property_name)))
                        .with(Bounds::next_vert(TOOL_CONTROL_SIZE))
                        .with(Hint::Class("tool-button".to_string()))
                        .with(vec![
                            Control::empty()
                                .with(Bounds::fill_all())
                                .with(image)
                        ]);

                    Some(control)
                });

            tool_set_controls.collect::<Vec<_>>()
        });

        // Create the main control binding
        let tool_sets               = tools.tool_sets.clone();
        let tools_for_selected_set  = tools.tools_for_selected_set();
        let viewmodel               = viewmodel.clone();

        BindRef::from(computed(move || {
            // Convert the tool sets into tools (with separators between each individual set)
            let tools_for_sets: Vec<_> = tool_sets.get().iter()
                .map(|toolset| {
                    let tools: Vec<_> = toolset.tools().iter()
                        .map(|tool| Self::make_tool(&tool.tool_name(), &viewmodel, &*images))
                        .collect();

                    tools
                }).fold(vec![], |mut result, new_items| {
                    // Separator between toolsets after the first set
                    if result.len() > 0 { result.push(Self::make_separator()); }

                    // Add the new items
                    result.extend(new_items.into_iter());

                    result
                });

            // Put the controls into a container
            Control::container()
                .with(Bounds::fill_all())
                .with(vec![
                    Control::container()
                        .with(Bounds::stretch_vert(1.0))
                        .with(tools_for_sets),
                    Control::container()
                        .with(Bounds::stretch_vert(1.0))
                        .with(tool_set_selector.get())
                ])
                .with(Appearance::Background(TOOLS_BACKGROUND))
        }))
    }

    ///
    /// Creates the image resources for this controller
    ///
    fn create_images(anim_model: &FloModel<Anim>) -> ResourceManager<Image> {
        let images  = ResourceManager::new();
        let tools   = anim_model.tools();

        // Load the tool images
        for tool_set in tools.tool_sets.get().iter() {
            // TODO: really want to be able to bind the tool images dynamically here 
            // (we can add extra sets by editing the model but the images won't load in: need to make the resource manager
            // dynamic to support this)
            for tool in tool_set.tools() {
                if let Some(image) = tool.image() {
                    // Give the image a name (the tool- suffix ensures that we can register other images without causing a clash)
                    let image_name      = format!("tool-{}", tool.tool_name());
                    let image_resource  = images.register(image);

                    // Assign a name to this tool
                    images.assign_name(&image_resource, &image_name);
                }
            }
        }

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
    fn make_tool(name: &str, viewmodel: &DynamicViewModel, images: &ResourceManager<Image>) -> Control {
        use self::ActionTrigger::*;

        // Decide if this is the selected tool
        let selected_tool   = viewmodel.get_property("SelectedTool");
        let tool_image      = images.get_named_resource(&format!("tool-{}", name));

        // The tool has a '-selected' binding that we use to cause it to highlight
        let compare_name            = String::from(name);
        let selected_property_name  = format!("{}-selected", name);
        let badged_property_name    = format!("{}-badged", name);

        // When the selected tool is set to the name of this tool, the selected property should be set to true
        viewmodel.set_computed(&selected_property_name, move || {
            let selected_tool = selected_tool.get().string().unwrap_or(String::from(""));
            PropertyValue::Bool(selected_tool == compare_name)
        });

        // When the effective tool is different from the selected tool, it displays a badge
        let selected_tool   = viewmodel.get_property("SelectedTool");
        let effective_tool  = viewmodel.get_property("EffectiveTool");
        let compare_name    = String::from(name);

        viewmodel.set_computed(&badged_property_name, move || {
            let selected_tool   = selected_tool.get().string().unwrap_or(String::from(""));
            let effective_tool  = effective_tool.get().string().unwrap_or(String::from(""));

            PropertyValue::Bool(selected_tool != effective_tool && effective_tool == compare_name)
        });

        // The control is just a button
        Control::button()
            .with((Click, name))
            .with(State::Badged(Property::Bind(badged_property_name)))
            .with(State::Selected(Property::Bind(selected_property_name)))
            .with(Bounds::next_vert(TOOL_CONTROL_SIZE))
            .with(Hint::Class("tool-button".to_string()))
            .with(vec![
                Control::empty()
                    .with(Bounds::fill_all())
                    .with(tool_image)
            ])
    }
}

impl<Anim: 'static+EditableAnimation+Animation> Controller for ToolboxController<Anim> {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    fn action(&self, action_id: &str, _action_parameter: &ActionParameter) {
        // Set the selected tool in the UI view model
        self.view_model.set_property("SelectedTool", PropertyValue::String(String::from(action_id)));

        // Update the animation view model with the newly selected tool
        self.anim_model.tools().choose_tool_with_name(action_id);
    }

    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
        Some(self.view_model.clone())
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(self.images.clone())
    }
}
