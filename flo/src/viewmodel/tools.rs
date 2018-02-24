use super::super::tools::*;
use super::super::standard_tools::*;

use ui::*;
use binding::*;
use animation::*;

use typemap::*;
use std::sync::*;

///
/// View model representing the currently selected and available tools
/// 
pub struct ToolViewModel<Anim: Animation> {
    // Binding that tracks the current tool activation state (changes to whatever the current tool uses)
    effective_tool_activated: Arc<Mutex<BindRef<ToolActivationState>>>,

    /// The currently selected tool
    pub selected_tool: Binding<Option<Arc<Tool2<GenericToolData, Anim>>>>,

    /// The ID of the pointer that's currently in use (device and pointer ID)
    pub current_pointer: Binding<(PaintDevice, i32)>,

    /// The tool that is in effect at the current moment (might change if the user chooses a different pointer)
    pub effective_tool: BindRef<Option<Arc<Tool2<GenericToolData, Anim>>>>,

    /// The tool sets available for selection
    pub tool_sets: Binding<Vec<Arc<ToolSet<Anim>>>>,
}

impl<Anim: Animation+'static> ToolViewModel<Anim> {
    ///
    /// Creates a new view model
    /// 
    pub fn new() -> ToolViewModel<Anim> {
        // Create the initial set of tools
        let default_tool_sets: Vec<Arc<ToolSet<Anim>>> = vec![
            Arc::new(SelectionTools::new()),
            Arc::new(PaintTools::new())
        ];

        // Create the bindings
        let effective_tool_activated    = bind(ToolActivationState::NeedsReactivation);
        let selected_tool               = bind(None);
        let tool_sets                   = bind(default_tool_sets);
        let current_pointer             = bind((PaintDevice::Mouse(MouseButton::Left), 0));

        // Finish up the object
        ToolViewModel {
            effective_tool:             Self::effective_tool(selected_tool.clone(), current_pointer.clone(), tool_sets.clone()),
            selected_tool:              selected_tool,
            tool_sets:                  tool_sets,
            current_pointer:            current_pointer,
            effective_tool_activated:   Arc::new(Mutex::new(BindRef::from(effective_tool_activated)))
        }
    }

    ///
    /// Returns a binding for the 'effective tool'
    /// 
    fn effective_tool(selected_tool: Binding<Option<Arc<Tool2<GenericToolData, Anim>>>>, current_pointer: Binding<(PaintDevice, i32)>, tool_sets: Binding<Vec<Arc<ToolSet<Anim>>>>) -> BindRef<Option<Arc<Tool2<GenericToolData, Anim>>>> {
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
    /// Calls activate on the effective tool if it is marked as inactive
    /// 
    pub fn activate_tool<'a>(&self, model: &ToolModel<'a, Anim>) {
        // Check if the tool has been activated
        let activation_state = self.effective_tool_activated.lock().unwrap().get();

        // If the tool has not been activated, mark it as active
        if activation_state != ToolActivationState::Activated {
            let effective_tool  = self.effective_tool.get();

            // Clear the tool state
            *model.tool_state.lock().unwrap() = SendMap::custom();

            // Mark the effective tool as active
            let tool_activation = effective_tool.map(|tool| tool.activate(model));
            
            if let Some(tool_activation) = tool_activation {
                // Tool needs reactivation if a different tool is selected
                let effective_tool_name = self.effective_tool.get().map(|tool| tool.tool_name()).unwrap_or("".to_string());
                let effective_tool      = BindRef::clone(&self.effective_tool);
                let tool_activation     = computed(move || {
                    if effective_tool.get().map(|tool| tool.tool_name()).as_ref() != Some(&effective_tool_name) {
                        ToolActivationState::NeedsReactivation
                    } else {
                        tool_activation.get()
                    }
                });

                // Re-activate the effective tool if this state changes
                *self.effective_tool_activated.lock().unwrap() = BindRef::from(tool_activation);
            }
        }
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
        self.selected_tool.clone().set(tool_with_name)
    }
}

impl<Anim: Animation> Clone for ToolViewModel<Anim> {
    fn clone(&self) -> ToolViewModel<Anim> {
        ToolViewModel {
            effective_tool_activated:   Arc::clone(&self.effective_tool_activated),
            selected_tool:              Binding::clone(&self.selected_tool),
            tool_sets:                  Binding::clone(&self.tool_sets),
            current_pointer:            Binding::clone(&self.current_pointer),
            effective_tool:             BindRef::clone(&self.effective_tool)
        }
    }
}