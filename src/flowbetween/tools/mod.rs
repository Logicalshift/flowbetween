use ui::*;
use ui::canvas::*;
use animation::*;

use std::sync::*;

mod tool_model;
mod tool_sets;
mod select;
mod adjust;
mod pan;
mod pencil;
mod ink;

pub use self::tool_model::*;
pub use self::tool_sets::*;
pub use self::select::*;
pub use self::adjust::*;
pub use self::pan::*;
pub use self::pencil::*;
pub use self::ink::*;

///
/// Trait implemented by tool objects
/// 
pub trait Tool<Anim: Animation> : Send+Sync {
    ///
    /// Retrieves the name of this tool
    /// 
    fn tool_name(&self) -> String;

    ///
    /// Retrieves the name of the image that is associated with this tool
    /// 
    fn image_name(&self) -> String;

    ///
    /// User is painting with this tool selected alongside a particular layer
    /// 
    fn paint<'a>(&self, model: &ToolModel<'a, Anim>, device: &PaintDevice, actions: &Vec<Painting>);
}

///
/// Equality so that tool objects can be referred to in bindings
/// 
impl<Anim: Animation> PartialEq for Tool<Anim> {
    fn eq(&self, other: &Tool<Anim>) -> bool {
        self.tool_name() == other.tool_name()
    }
}

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
    fn tools(&self) -> Vec<Arc<Tool<Anim>>>;
}

///
/// Equality so that tool objects can be referred to in bindings
/// 
impl<Anim: Animation> PartialEq for ToolSet<Anim> {
    fn eq(&self, other: &ToolSet<Anim>) -> bool {
        self.set_name() == other.set_name()
    }
}
