use super::graphics::*;

///
/// A layer represents a renderable plane in a frame
///
pub trait Layer {
    ///
    /// Retrieves the rendering commands required to render this layer
    ///
    fn render_commands(&self) -> Box<Iterator<Item = GraphicsCommand>>;
}
