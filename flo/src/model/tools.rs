use super::super::tools::*;
use super::super::model::*;
use super::super::standard_tools::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;
use std::collections::HashMap;

///
/// View model representing the currently selected and available tools
///
pub struct ToolModel<Anim: Animation> {
    /// The currently selected tool
    pub selected_tool: Binding<Option<Arc<FloTool<Anim>>>>,

    /// The ID of the pointer that's currently in use (device and pointer ID)
    pub current_pointer: Binding<(PaintDevice, i32)>,

    /// The tool that is in effect at the current moment (might change if the user chooses a different pointer)
    pub effective_tool: BindRef<Option<Arc<FloTool<Anim>>>>,

    /// The tool sets available for selection
    pub tool_sets: Binding<Vec<Arc<dyn ToolSet<Anim>>>>,

    /// The models for each tool in the toolsets
    tool_models: Arc<Mutex<HashMap<String, Arc<GenericToolModel>>>>
}

impl<Anim: EditableAnimation+Animation+'static> ToolModel<Anim> {
    ///
    /// Creates a new view model
    ///
    pub fn new() -> ToolModel<Anim> {
        // Create the initial set of tools
        let default_tool_sets: Vec<Arc<dyn ToolSet<Anim>>> = vec![
            Arc::new(SelectionTools::new()),
            Arc::new(PaintTools::new())
        ];

        // Create the bindings
        let selected_tool               = bind(None);
        let tool_sets                   = bind(default_tool_sets);
        let current_pointer             = bind((PaintDevice::Mouse(MouseButton::Left), 0));
        let tool_models                 = Arc::new(Mutex::new(HashMap::new()));
        let effective_tool              = Self::effective_tool(selected_tool.clone(), current_pointer.clone(), tool_sets.clone());

        // Finish up the object
        ToolModel {
            effective_tool:             effective_tool,
            selected_tool:              selected_tool,
            tool_sets:                  tool_sets,
            current_pointer:            current_pointer,
            tool_models:                tool_models
        }
    }

    ///
    /// Returns the model for the specified tool
    ///
    pub fn model_for_tool(&self, tool: &FloTool<Anim>, model: Arc<FloModel<Anim>>) -> Arc<GenericToolModel> {
        self.tool_models.lock().unwrap()
            .entry(tool.tool_name())
            .or_insert_with(move || Arc::new(tool.create_model(model)))
            .clone()
    }

    ///
    /// Returns a binding for the 'effective tool'
    ///
    fn effective_tool(selected_tool: Binding<Option<Arc<FloTool<Anim>>>>, current_pointer: Binding<(PaintDevice, i32)>, tool_sets: Binding<Vec<Arc<dyn ToolSet<Anim>>>>) -> BindRef<Option<Arc<FloTool<Anim>>>> {
        let effective_tool = computed(move || {
            let (device, _pointer_id) = current_pointer.get();

            match device {
                // For the mouse, we always use the selected tool
                PaintDevice::Mouse(_) => selected_tool.get(),
                PaintDevice::Other => selected_tool.get(),

                // The eraser defaults to the erase tool if it's available
                // TODO: or whatever was selected using the eraser
                PaintDevice::Eraser => {
                    let mut erase_tool = selected_tool.get();

                    let tool_sets = tool_sets.get();
                    for set in tool_sets {
                        let tools = set.tools();
                        for tool in tools {
                            if &tool.tool_name() == "Eraser" {
                                erase_tool = Some(tool);
                            }
                        }
                    }

                    erase_tool
                },

                // Other pointers default to the selected tool but will use whatever tool they were last used to select if there is one
                _ => selected_tool.get()
            }
        });

        BindRef::from(effective_tool)
    }

    ///
    /// Finds the tool with the specified name and marks it as active
    ///
    pub fn choose_tool_with_name(&self, name: &str) {
        let mut tool_with_name = None;

        // Search all of the toolsets for a tool matching the specified name
        for set in self.tool_sets.get() {
            for tool in set.tools() {
                if &tool.tool_name() == name {
                    tool_with_name = Some(tool);
                }
            }
        }

        // Set as the selected tool
        self.selected_tool.set(tool_with_name)
    }
}

impl<Anim: Animation> Clone for ToolModel<Anim> {
    fn clone(&self) -> ToolModel<Anim> {
        ToolModel {
            selected_tool:              Binding::clone(&self.selected_tool),
            tool_sets:                  Binding::clone(&self.tool_sets),
            current_pointer:            Binding::clone(&self.current_pointer),
            effective_tool:             BindRef::clone(&self.effective_tool),
            tool_models:                Arc::clone(&self.tool_models)
        }
    }
}
