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
    /// The ID of the pointer that's currently in use (device and pointer ID)
    pub current_pointer: Binding<(PaintDevice, i32)>,

    /// The tool that is in effect at the current moment (might change if the user chooses a different pointer)
    pub effective_tool: BindRef<Option<Arc<FloTool<Anim>>>>,

    /// The tool sets available for selection
    pub tool_sets: Binding<Vec<Arc<dyn ToolSet<Anim>>>>,

    /// The name of the currently selected toolset
    pub selected_tool_set: Binding<Option<ToolSetId>>,

    /// The selected tool for the toolset with the specified name
    selected_tool_for_set: Arc<Mutex<HashMap<ToolSetId, Binding<Option<Arc<FloTool<Anim>>>>>>>,

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
        let selected_tool_set           = bind(None);
        let selected_tool_for_set       = Arc::new(Mutex::new(HashMap::new()));
        let tool_sets                   = bind(default_tool_sets);
        let current_pointer             = bind((PaintDevice::Mouse(MouseButton::Left), 0));
        let tool_models                 = Arc::new(Mutex::new(HashMap::new()));
        let effective_tool              = Self::effective_tool(selected_tool_set.clone(), selected_tool_for_set.clone(), current_pointer.clone(), tool_sets.clone());

        // Finish up the object
        ToolModel {
            effective_tool:             effective_tool,
            tool_sets:                  tool_sets,
            selected_tool_set:          selected_tool_set,
            selected_tool_for_set:      selected_tool_for_set,
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
    fn effective_tool(selected_tool_set: Binding<Option<ToolSetId>>, selected_tool_for_set: Arc<Mutex<HashMap<ToolSetId, Binding<Option<Arc<FloTool<Anim>>>>>>>, current_pointer: Binding<(PaintDevice, i32)>, tool_sets: Binding<Vec<Arc<dyn ToolSet<Anim>>>>) -> BindRef<Option<Arc<FloTool<Anim>>>> {
        let selected_tool = computed(move || {
            selected_tool_set.get()
                .and_then(|selected_tool_set| selected_tool_for_set.lock().unwrap().get(&selected_tool_set)
                    .and_then(|tool| tool.get()))
        });

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
    /// Returns a binding that indicates which tools are available for the currently selected toolset
    ///
    pub fn tools_for_selected_set(&self) -> impl Bound<Vec<Arc<FloTool<Anim>>>> {
        let selected_tool_set   = self.selected_tool_set.clone();
        let tool_sets           = self.tool_sets.clone();

        computed(move || {
            let selected_tool_set_id    = selected_tool_set.get();
            let tool_sets               = tool_sets.get();
            let tool_set                = tool_sets.iter().filter(|set| Some(set.id()) == selected_tool_set_id).nth(0);

            tool_set
                .map(|tool_set| tool_set.tools())
                .unwrap_or_else(|| vec![])
        })
    }

    ///
    /// Finds the tool with the specified name and marks it as active
    ///
    pub fn choose_tool_with_name(&self, name: &str) {
        // Search all of the toolsets for a tool matching the specified name
        for set in self.tool_sets.get() {
            for tool in set.tools() {
                if &tool.tool_name() == name {
                    if self.selected_tool_set.get() != Some(set.id()) {
                        self.selected_tool_set.set(Some(set.id()));
                    }

                    self.selected_tool_for_set.lock().unwrap()
                        .entry(set.id())
                        .or_insert_with(|| bind(None))
                        .set(Some(tool));
                }
            }
        }
    }
}

impl<Anim: Animation> Clone for ToolModel<Anim> {
    fn clone(&self) -> ToolModel<Anim> {
        ToolModel {
            tool_sets:                  Binding::clone(&self.tool_sets),
            current_pointer:            Binding::clone(&self.current_pointer),
            effective_tool:             BindRef::clone(&self.effective_tool),
            selected_tool_set:          Binding::clone(&self.selected_tool_set),
            selected_tool_for_set:      Arc::clone(&self.selected_tool_for_set),
            tool_models:                Arc::clone(&self.tool_models)
        }
    }
}
