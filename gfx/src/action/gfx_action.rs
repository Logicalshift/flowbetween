use super::identities::*;
use super::render_target_type::*;
use crate::buffer::*;

///
/// Represents an action for a `gfx` target
///
#[derive(Clone, PartialEq, Debug)]
pub enum GfxAction {
    ///
    /// Creates a vertex buffer with the specified 2D vertices in it (replacing any existing buffer)
    ///
    CreateVertex2DBuffer(VertexBufferId, Vec<Vertex2D>),

    ///
    /// Frees an existing vertex buffer
    ///
    FreeVertexBuffer(VertexBufferId),

    ///
    /// Creates a new render target of the specified size, as the specified texture
    ///
    CreateRenderTarget(RenderTargetId, TextureId, usize, usize, RenderTargetType),

    ///
    /// Frees up an existing render target
    ///
    FreeRenderTarget(RenderTargetId),

    ///
    /// Creates an 8-bit RGBA texture of the specified size
    ///
    CreateTextureRgba(TextureId, usize, usize),

    ///
    /// Loads byte data into the specifed texture, at the specified offset
    ///
    LoadTextureData(TextureId, usize, Vec<u8>)
}
