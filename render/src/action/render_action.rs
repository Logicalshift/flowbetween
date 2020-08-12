use super::identities::*;
use super::render_target_type::*;
use super::color::*;
use super::blend_mode::*;
use super::shader_type::*;

use crate::buffer::*;

use std::ops::{Range};

///
/// Represents an action for a render target
///
#[derive(Clone, PartialEq, Debug)]
pub enum RenderAction {
    ///
    /// Sets the transformation matrix to use for future renderings
    ///
    SetTransform(Matrix),

    ///
    /// Creates a vertex buffer with the specified 2D vertices in it (replacing any existing buffer)
    ///
    CreateVertex2DBuffer(VertexBufferId, Vec<Vertex2D>),

    ///
    /// Creates an index buffer with the specified 2D vertices in it (replacing any existing buffer)
    ///
    CreateIndexBuffer(IndexBufferId, Vec<u16>),

    ///
    /// Frees an existing vertex buffer
    ///
    FreeVertexBuffer(VertexBufferId),

    ///
    /// Frees an existing index buffer
    ///
    FreeIndexBuffer(IndexBufferId),

    ///
    /// Sets the blend mode for future drawing operations (SourceOver is the default)
    ///
    BlendMode(BlendMode),

    ///
    /// Creates a new render target of the specified size, as the specified texture
    ///
    CreateRenderTarget(RenderTargetId, TextureId, usize, usize, RenderTargetType),

    ///
    /// Frees up an existing render target
    ///
    FreeRenderTarget(RenderTargetId),

    ///
    /// Send future rendering instructions to the specified render target
    ///
    SelectRenderTarget(RenderTargetId),

    ///
    /// Send future rendering instructions to the main frame buffer
    ///
    RenderToFrameBuffer,

    ///
    /// Display the current frame buffer on-screen
    ///
    ShowFrameBuffer,

    ///
    /// Renders the specified framebuffer to the current framebuffer
    ///
    DrawFrameBuffer(RenderTargetId, i32, i32),

    ///
    /// Creates an 8-bit BGRA texture of the specified size
    ///
    CreateTextureBgra(TextureId, usize, usize),

    ///
    /// Frees up an existing texture
    ///
    FreeTexture(TextureId),

    ///
    /// Clears the current render target to the specified colour
    ///
    Clear(Rgba8),

    ///
    /// Uses the specified shader
    ///
    UseShader(ShaderType),

    ///
    /// Renders triangles from a vertex buffer (with no texture)
    ///
    /// Parameters are the range of vertices to use
    ///
    DrawTriangles(VertexBufferId, Range<usize>),

    ///
    /// Renders triangles using an index buffer
    ///
    DrawIndexedTriangles(VertexBufferId, IndexBufferId, usize)
}
