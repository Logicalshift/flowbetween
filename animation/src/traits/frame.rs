use canvas::*;

use std::time::Duration;

///
/// Represents a single frame in a layer of an animation
///
pub trait Frame : Send+Sync {
    ///
    /// Time index of this frame
    /// 
    fn time_index(&self) -> Duration;

    ///
    /// Renders this frame to a particular graphics context
    ///
    fn render_to(&self, gc: &mut GraphicsPrimitives);
}
