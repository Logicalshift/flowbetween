use super::buffer::*;
use super::vertex_array::*;

use crate::action::*;
use crate::buffer::*;

///
/// The data associated with a render target
///
#[derive(Clone)]
struct RenderTarget {
}

///
/// OpenGL action renderer
///
pub struct GlRenderer {
    /// Definition of the Vertex2D array type
    vertex_2d_array: VertexArray,

    // The buffers allocated to this renderer
    buffers: Vec<Option<Buffer>>
}

impl GlRenderer {
    ///
    /// Creates a new renderer that will render to the specified device and factory
    ///
    pub fn new() -> GlRenderer {
        GlRenderer {
            vertex_2d_array:    Vertex2D::define_vertex_array(),
            buffers:            vec![]
        }
    }

    ///
    /// Performs rendering of the specified actions to this device target
    ///
    pub fn render<Actions: IntoIterator<Item=GfxAction>>(&mut self, actions: Actions) {
        for action in actions {
            use self::GfxAction::*;

            match action {
                CreateVertex2DBuffer(id, vertices)                                      => { self.create_vertex_buffer_2d(id, vertices); }
                FreeVertexBuffer(id)                                                    => { self.free_vertex_buffer(id); }
                CreateRenderTarget(render_id, texture_id, width, height, render_type)   => { self.create_render_target(render_id, texture_id, width, height, render_type); }
                FreeRenderTarget(render_id)                                             => { self.free_render_target(render_id); }
                SelectRenderTarget(render_id)                                           => { self.select_render_target(render_id); }
                RenderToFrameBuffer                                                     => { self.select_main_frame_buffer(); }
                ShowFrameBuffer                                                         => { /* This doesn't double-buffer so nothing to do */ }
                CreateTextureBgra(texture_id, width, height)                            => { self.create_bgra_texture(texture_id, width, height); }
                FreeTexture(texture_id)                                                 => { self.free_texture(texture_id); }
                Clear(color)                                                            => { self.clear(color); }
            }
        }
    }

    ///
    /// Clears the current render target
    ///
    fn clear(&mut self, Rgba8([r, g, b, a]): Rgba8) {
        let r = (r as f32)/255.0;
        let g = (g as f32)/255.0;
        let b = (b as f32)/255.0;
        let a = (a as f32)/255.0;

        unsafe { gl::ClearBufferfv(gl::COLOR, 0, &[r, g, b, a][0]); }
    }

    ///
    /// Creates a 2D vertex buffer
    ///
    fn create_vertex_buffer_2d(&mut self, VertexBufferId(id): VertexBufferId, vertices: Vec<Vertex2D>) {
        // Extend the buffers array as needed
        if id >= self.buffers.len() {
            self.buffers.extend((self.buffers.len()..(id+1))
                .into_iter()
                .map(|_| None));
        }

        // Create a buffer containing these vertices
        let mut buffer = Buffer::new();
        buffer.static_draw(&vertices);

        // Store in the buffers collections
        self.buffers[id] = Some(buffer);
    }

    ///
    /// Frees the vertex buffer with the specified ID
    ///
    fn free_vertex_buffer(&mut self, VertexBufferId(id): VertexBufferId) {
        self.buffers[id] = None;
    }

    ///
    /// Creates a new BGRA texture
    ///
    fn create_bgra_texture(&mut self, TextureId(id): TextureId, width: usize, height: usize) {
    }

    ///
    /// Releases an existing render target
    ///
    fn free_texture(&mut self, TextureId(texture_id): TextureId) {
    }

    ///
    /// Creates a new render target
    ///
    fn create_render_target(&mut self, RenderTargetId(render_id): RenderTargetId, TextureId(texture_id): TextureId, width: usize, height: usize, render_type: RenderTargetType) {
    }

    ///
    /// Chooses which buffer rendering instructions will be sent to
    ///
    fn select_render_target(&mut self, RenderTargetId(render_id): RenderTargetId) {
    }

    ///
    /// Sends rendering instructions to the primary frame buffer for display
    ///
    fn select_main_frame_buffer(&mut self) {
    }

    ///
    /// Releases an existing render target
    ///
    fn free_render_target(&mut self, RenderTargetId(render_id): RenderTargetId) {
    }

    ///
    /// Flushes all changes to the device
    ///
    pub fn flush(&mut self) {
    }
}
