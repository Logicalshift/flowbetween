use ui::*;
use animation::*;

mod tool_action;
mod brush_preview_action;
mod overlay_action;
mod tool_input;
mod tool_trait;
mod tool_set;
mod generic_tool;
mod tool_runner;

pub use self::tool_action::*;
pub use self::brush_preview_action::*;
pub use self::overlay_action::*;
pub use self::tool_input::*;
pub use self::tool_trait::*;
pub use self::tool_set::*;
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
