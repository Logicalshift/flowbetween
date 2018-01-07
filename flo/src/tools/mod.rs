use ui::*;
use binding::*;
use animation::*;

use std::sync::*;

mod tool_model;
mod tool_sets;
mod select;
mod adjust;
mod pan;
mod pencil;
mod ink;
mod eraser;

pub use self::tool_model::*;
pub use self::tool_sets::*;
pub use self::select::*;
pub use self::adjust::*;
pub use self::pan::*;
pub use self::pencil::*;
pub use self::ink::*;
pub use self::eraser::*;

///
/// Converts a UI Painting struct to a BrushPoint
/// 
pub fn brush_point_from_painting(painting: &Painting) -> BrushPoint {
    BrushPoint {
        position: painting.location,
        pressure: painting.pressure
    }
}

///
/// Trait indicating the current activation state of a tool
///
#[derive(Clone, Copy, PartialEq)]
pub enum ToolActivationState {
    /// Tool is currently activated and doesn't need reactivation
    Activated,

    /// Tool needs to be reactivated before it can be re-used
    NeedsReactivation
}

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
    /// Retrieves the menu controller to use for adjusting this tool
    /// 
    fn menu_controller_name(&self) -> String { "".to_string() }

    ///
    /// Activates this tool (called before this tool performs any other actions like paint)
    /// 
    /// The return value is a binding that indicates whether or not this tool
    /// needs reactivating.
    ///
    fn activate<'a>(&self, _model: &ToolModel<'a, Anim>) -> BindRef<ToolActivationState> {
        BindRef::from(bind(ToolActivationState::Activated))
    }

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
