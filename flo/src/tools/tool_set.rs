use super::generic_tool::*;

use flo_animation::*;

use std::sync::*;

/// An identifier for a toolset
#[derive(Clone, PartialEq, Eq, PartialOrd, Hash, Debug)]
pub struct ToolSetId(pub String);

///
/// Represents a grouped set of tools
///
pub trait ToolSet<Anim: Animation>: Send+Sync {
    ///
    /// The ID for this toolset
    ///
    fn id(&self) -> ToolSetId;

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
