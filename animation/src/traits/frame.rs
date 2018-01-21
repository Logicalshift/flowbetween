use canvas::*;

use std::time::Duration;
use std::any::Any;

///
/// Represents a single frame in a layer of an animation
///
pub trait Frame : Send+Sync+Any {
    ///
    /// Time index of this frame relative to its keyframe
    /// 
    fn time_index(&self) -> Duration;

    ///
    /// Renders this frame to a particular graphics context
    ///
    fn render_to(&self, gc: &mut GraphicsPrimitives);
}
