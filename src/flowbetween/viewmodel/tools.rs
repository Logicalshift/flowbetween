use super::super::tools::*;

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

        // Finish up the object
        ToolViewModel {
            current_tool:           current_tool,
            tool_sets:              tool_sets,
            current_tool_activated: Arc::new(Mutex::new(Box::new(current_tool_activated)))
        }
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
                let current_tool_name   = self.current_tool.get().map(|tool| tool.tool_name()).unwrap_or("".to_string());
                let current_tool        = Binding::clone(&self.current_tool);
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
            tool_sets:              Binding::clone(&self.tool_sets)
        }
    }
}