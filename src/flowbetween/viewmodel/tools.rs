use super::super::tools::*;

use binding::*;
use animation::*;

use std::sync::*;

///
/// View model representing the currently selected and available tools
/// 
pub struct ToolViewModel<Anim: Animation> {
    /// Whether or not activate has been called for the current tool
    /// 
    /// Set this to false if the current tool's state is changed such
    /// that the activate call needs to be made. Changing the tool
    /// will always set this to false, but things like changing the
    /// active layer may also make the activate call necessary.
    /// 
    /// TODO: maybe make this computed from the list of properties
    /// that matter - ie, tool name, selected layer, etc.
    current_tool_activated: Binding<bool>,

    /// The currently selected tool
    current_tool: Binding<Option<Arc<Tool<Anim>>>>,

    /// The tool sets available for selection
    tool_sets: Binding<Vec<Arc<ToolSet<Anim>>>>,

    /// Lifetimes oft he actions performed when the current tool changes
    monitors: Arc<Mutex<Box<Releasable>>>
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
        let current_tool_activated  = bind(false);
        let current_tool            = bind(None);
        let tool_sets               = bind(default_tool_sets);

        // Whenever the current tool changes, mark it as not activated
        let mut activated   = current_tool_activated.clone();
        let monitors        = current_tool.when_changed(notify(move || activated.set(false)));

        // Finish up the object
        ToolViewModel {
            current_tool:           current_tool,
            tool_sets:              tool_sets,
            current_tool_activated: current_tool_activated,
            monitors:               Arc::new(Mutex::new(monitors))
        }
    }

    ///
    /// Calls activate on the current tool if it is marked as inactive
    /// 
    pub fn activate_tool<'a>(&self, model: &ToolModel<'a, Anim>) {
        // TODO: currently we only deactivate the active tool if the chosen tool changes
        // TODO: we should probably have a sensible way to deactivate the tool if something else relevant changes
        // TODO: for example, the selected layer or brush properties

        // Check if the tool has been activated
        let is_active = self.current_tool_activated.get();

        // If the tool has not been activated, mark it as active
        if !is_active {
            // Mark the current tool as active
            self.current_tool.get().map(|tool| tool.activate(model));
            
            // Tool is activated
            self.current_tool_activated.clone().set(true);
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

    ///
    /// Retrieves the tool sets binding
    /// 
    pub fn tool_sets(&self) -> Binding<Vec<Arc<ToolSet<Anim>>>> {
        Binding::clone(&self.tool_sets)
    }

    ///
    /// Retrieves the currently selected tool binding
    /// 
    pub fn current_tool(&self) -> Binding<Option<Arc<Tool<Anim>>>> {
        Binding::clone(&self.current_tool)
    }
}

impl<Anim: Animation> Clone for ToolViewModel<Anim> {
    fn clone(&self) -> ToolViewModel<Anim> {
        ToolViewModel {
            current_tool_activated: Binding::clone(&self.current_tool_activated),
            current_tool:           Binding::clone(&self.current_tool),
            tool_sets:              Binding::clone(&self.tool_sets),
            monitors:               Arc::clone(&self.monitors)
        }
    }
}