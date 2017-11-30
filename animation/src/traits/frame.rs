use ui::canvas::*;

///
/// Represents a single frame in a layer of an animation
///
pub trait Frame {
    ///
    /// Time index of this frame, in nanoseconds
    /// 
    fn time_index(&self) -> u64;

    ///
    /// Renders this frame to a particular graphics context
    ///
    fn render_to(&self, gc: &mut GraphicsContext);
}
