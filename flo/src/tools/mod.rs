use ui::*;
use animation::*;

use std::sync::*;

mod tool_model;

mod tool_action;
mod brush_preview_action;
mod tool_input;
mod tool_trait;
mod generic_tool;
mod tool_runner;

pub use self::tool_model::*;

pub use self::tool_action::*;
pub use self::brush_preview_action::*;
pub use self::tool_input::*;
pub use self::tool_trait::*;
pub use self::generic_tool::*;
pub use self::tool_runner::*;

///
/// Converts a UI Painting struct to a BrushPoint
/// 
pub fn raw_point_from_painting(painting: &Painting) -> RawPoint {
    RawPoint {
        position:   painting.location,
        tilt:       (painting.tilt_x, painting.tilt_y),
        pressure:   painting.pressure
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
impl<Anim: Animation> PartialEq for ToolSet<Anim> {
    fn eq(&self, other: &ToolSet<Anim>) -> bool {
        self.set_name() == other.set_name()
    }
}
