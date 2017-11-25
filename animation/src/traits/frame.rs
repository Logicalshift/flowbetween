use ui::canvas::*;
use super::attributes::*;

///
/// Represents a single frame in a layer of an animation
///
pub trait Frame : HasAttributes {
    ///
    /// Renders this frame to a particular graphics context
    ///
    fn render_to(&self, gc: &mut GraphicsContext);
}
