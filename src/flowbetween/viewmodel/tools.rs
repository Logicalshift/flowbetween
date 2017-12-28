use super::super::tools::*;

use binding::*;
use animation::*;

use std::sync::*;

///
/// View model representing the currently selected and available tools
/// 
pub struct ToolViewModel<Anim: Animation> {
    /// The currently selected tool
    current_tool: Binding<Option<Arc<Tool<Anim>>>>,

    /// The tool sets available for selection
    tool_sets: Binding<Vec<Arc<ToolSet<Anim>>>>,
}

impl<Anim: Animation+'static> ToolViewModel<Anim> {
    ///
    /// Creates a new view model
    /// 
    pub fn new() -> ToolViewModel<Anim> {
        let default_tool_sets: Vec<Arc<ToolSet<Anim>>> = vec![
            Arc::new(SelectionTools::new()),
            Arc::new(PaintTools::new())
        ];

        ToolViewModel {
            current_tool:   bind(None),
            tool_sets:      bind(default_tool_sets),
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
            current_tool:   Binding::clone(&self.current_tool),
            tool_sets:      Binding::clone(&self.tool_sets)
        }
    }
}