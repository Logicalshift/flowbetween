///
/// Represents a single frame in a layer of an animation
///
pub trait Frame {
    ///
    /// Retrieves the rendering commands required to render this layer
    ///
    fn render_commands(&self) -> Box<Iterator<Item = GraphicsCommand>>;
}
