use crate::buffer::*;

///
/// Represents an action for a `gfx` target
///
pub enum GfxAction {
    ///
    /// Creates a vertex buffer with the specified 2D vertices in it (replacing any existing buffer)
    ///
    CreateVertex2DBuffer(usize, Vec<Vertex2D>),

    ///
    /// Frees an existing vertex buffer
    ///
    FreeVertexBuffer(usize)
}
