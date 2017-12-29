use super::super::tools::*;

use ui::*;
use binding::*;
use animation::*;

use std::sync::*;

///
/// View model representing the currently selected and available tools
/// 
pub struct ToolViewModel<Anim: Animation> {
    // Binding that tracks the current tool activation state (changes to whatever the current tool uses)
    current_tool_activated: Arc<Mutex<Box<Bound<ToolActivationState>>>>,

    /// The currently selected tool
    pub current_tool: Binding<Option<Arc<Tool<Anim>>>>,

    /// The ID of the pointer that's currently in use (device and pointer ID)
    pub current_pointer: Binding<(PaintDevice, i32)>,

    /// The tool that is in effect at the current moment (might change if the user chooses a different pointer)
    pub effective_tool: Arc<Bound<Option<Arc<Tool<Anim>>>>>,

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
        let current_tool_activated  = bind(ToolActivationState::NeedsReactivation);
        let current_tool            = bind(None);
        let tool_sets               = bind(default_tool_sets);
        let current_pointer         = bind((PaintDevice::Mouse(MouseButton::Left), 0));

        // Finish up the object
        ToolViewModel {
            effective_tool:         Self::effective_tool(current_tool.clone(), current_pointer.clone(), tool_sets.clone()),
            current_tool:           current_tool,
            tool_sets:              tool_sets,
            current_pointer:        current_pointer,
            current_tool_activated: Arc::new(Mutex::new(Box::new(current_tool_activated)))
        }
    }

    ///
    /// Returns a binding for the 'effective tool'
    /// 
    fn effective_tool(current_tool: Binding<Option<Arc<Tool<Anim>>>>, current_pointer: Binding<(PaintDevice, i32)>, tool_sets: Binding<Vec<Arc<ToolSet<Anim>>>>) -> Arc<Bound<Option<Arc<Tool<Anim>>>>> {
        let effective_tool = computed(move || {
            let (device, _pointer_id) = current_pointer.get();

            match device {
                // For the mouse, we always use the current tool
                PaintDevice::Mouse(_) => current_tool.get(),
                PaintDevice::Other => current_tool.get(),

                // The eraser defaults to the erase tool if it's available
                // TODO: or whatever was selected using the eraser
                PaintDevice::Eraser => {
                    let mut erase_tool = current_tool.get();

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

                // Other pointers default to the current tool but will use whatever tool they were last used to select if there is one
                _ => current_tool.get()
            }
        });

        Arc::new(effective_tool)
    }

    ///
    /// Calls activate on the current tool if it is marked as inactive
    /// 
    pub fn activate_tool<'a>(&self, model: &ToolModel<'a, Anim>) {
        // Check if the tool has been activated
        let activation_state = self.current_tool_activated.lock().unwrap().get();

        // If the tool has not been activated, mark it as active
        if activation_state != ToolActivationState::Activated {
            let current_tool    = self.current_tool.get();

            // Mark the current tool as active
            let tool_activation = current_tool.map(|tool| tool.activate(model));
            
            if let Some(tool_activation) = tool_activation {
                // Tool needs reactivation if a different tool is selected
                let current_tool_name   = self.effective_tool.get().map(|tool| tool.tool_name()).unwrap_or("".to_string());
                let current_tool        = Arc::clone(&self.effective_tool);
                let tool_activation     = computed(move || {
                    if current_tool.get().map(|tool| tool.tool_name()).as_ref() != Some(&current_tool_name) {
                        ToolActivationState::NeedsReactivation
                    } else {
                        tool_activation.get()
                    }
                });

                // Re-activate the current tool if this state changes
                *self.current_tool_activated.lock().unwrap() = Box::new(tool_activation);
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

        // Set as the current tool
        self.current_tool.clone().set(tool_with_name)
    }
}

impl<Anim: Animation> Clone for ToolViewModel<Anim> {
    fn clone(&self) -> ToolViewModel<Anim> {
        ToolViewModel {
            current_tool_activated: Arc::clone(&self.current_tool_activated),
            current_tool:           Binding::clone(&self.current_tool),
            tool_sets:              Binding::clone(&self.tool_sets),
            current_pointer:        Binding::clone(&self.current_pointer),
            effective_tool:         Arc::clone(&self.effective_tool)
        }
    }
}