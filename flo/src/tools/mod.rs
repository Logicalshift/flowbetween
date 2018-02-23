use ui::*;
use binding::*;
use animation::*;

use std::sync::*;

mod tool_model;

mod tool_action;
mod tool_input;
mod tool_trait;

pub use self::tool_model::*;

pub use self::tool_action::*;
pub use self::tool_input::*;
pub use self::tool_trait::*;
 
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
/// Trait implemented by tool objects
/// 
/// TODO: the 'I/O' model we use for canvases and the edit log kind of implies that
/// perhaps a tool should be a method that maps a series of input actions to
/// output actions rather than using a stateful model. (Brush strokes in particular
/// just queue all the raw points up so we kind of get this effect anyway)
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
