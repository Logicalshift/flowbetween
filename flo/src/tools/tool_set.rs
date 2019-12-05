use super::generic_tool::*;

use flo_animation::*;

use std::sync::*;

///
/// Represents a grouped set of tools
///
pub trait ToolSet<Anim: Animation>: Send+Sync {
    ///
    /// Retrieves the name of this tool set
    ///
    fn set_name(&self) -> String;

    ///
    /// Retrieves the tools in this set
    ///
    fn tools(&self) -> Vec<Arc<FloTool<Anim>>>;
}

///
/// Equality so that tool objects can be referred to in bindings
///
impl<Anim: Animation> PartialEq for dyn ToolSet<Anim> {
    fn eq(&self, other: &dyn ToolSet<Anim>) -> bool {
        self.set_name() == other.set_name()
    }
}
