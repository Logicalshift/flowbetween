use crate::action::*;
use crate::buffer::*;

use gfx;
use gfx::format;
use gfx::handle;
use gfx::memory;
use gfx::texture;
use gfx::traits::{FactoryExt};

///
/// The data associated with a render target
///
#[derive(Clone)]
struct RenderTarget<Device> 
where   Device:     gfx::Device {
    /// The render target view this will be rendering to
    render_target: handle::RenderTargetView<Device::Resources, format::Bgra8>,
}

///
/// Renders GFX actions to a GFX device
///
pub struct Renderer<Device, Factory>
where   Device:     gfx::Device,
        Factory:    gfx::Factory<Device::Resources> {
    /// The render device
    device:  Device,

    // The device factory
    factory: Factory,

    /// The command buffer for this renderer
    encoder: gfx::Encoder<Device::Resources, Device::CommandBuffer>,

    /// The currently selected render target
    active_render_target: usize,

    /// The 'main' depth stencil
    main_depth_stencil: Option<handle::DepthStencilView<Device::Resources, format::DepthStencil>>,

    /// Render targets created for this renderer
    render_targets: Vec<Option<RenderTarget<Device>>>,

    /// The vertex buffers that have been allocated (indexed by ID)
    vertex_buffers_2d: Vec<Option<handle::Buffer<Device::Resources, Vertex2D>>>,

    /// The BGRA format textures for this renderer
    bgra_textures: Vec<Option<handle::Texture<Device::Resources, format::B8_G8_R8_A8>>>,

    /// The RGBA format textures for this renderer
    rgba_textures: Vec<Option<handle::Texture<Device::Resources, format::R8_G8_B8_A8>>>
}

impl<Device, Factory> Renderer<Device, Factory>
where   Device:                 gfx::Device,
        Factory:                gfx::Factory<Device::Resources> {
    ///
    /// Creates a new renderer that will render to the specified device and factory
    ///
    pub fn new(
        device:             Device, 
        factory:            Factory, 
        encoder:            gfx::Encoder<Device::Resources, Device::CommandBuffer>) -> Renderer<Device, Factory> {
        Renderer {
            device:                 device,
            factory:                factory,
            encoder:                encoder,
            active_render_target:   0,
            main_depth_stencil:     None,
            render_targets:         vec![None],
            vertex_buffers_2d:      vec![],
            bgra_textures:          vec![],
            rgba_textures:          vec![]
        }
    }

    ///
    /// Updates the render target to use as the 'main' render target for this renderer
    ///
    pub fn set_main_render_target(&mut self, main_render_target: gfx::handle::RenderTargetView<Device::Resources, format::Bgra8>, main_depth_stencil: gfx::handle::DepthStencilView<Device::Resources, format::DepthStencil>) {
        self.render_targets[0]  = Some(RenderTarget { render_target: main_render_target });
        self.main_depth_stencil = Some(main_depth_stencil);
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
    fn clear(&mut self, color: Rgba8) {
        // Convert the colour to f32 values
        let Rgba8([r, g, b, a]) = color;
        let (r, g, b, a)        = ((r as f32)/255.0, (g as f32)/255.0, (b as f32)/255.0, (a as f32)/255.0);

        // Fetch the encoder
        let encoder             = &mut self.encoder;

        // Send to the current render target
        self.render_targets[self.active_render_target].as_ref().map(|render_target| {
            encoder.clear(&render_target.render_target, [r, g, b, a]);
        });
    }

    ///
    /// Creates a 2D vertex buffer
    ///
    fn create_vertex_buffer_2d(&mut self, VertexBufferId(id): VertexBufferId, vertices: Vec<Vertex2D>) {
        // Extend the vertex handle list if needed
        if self.vertex_buffers_2d.len() <= id {
            self.vertex_buffers_2d.extend((self.vertex_buffers_2d.len()..(id+1))
                .into_iter().map(|_| None));
        }

        // Create a new buffer
        self.vertex_buffers_2d[id] = None;

        let new_buffer = self.factory.create_vertex_buffer(&vertices);
        self.vertex_buffers_2d[id] = Some(new_buffer);
    }

    ///
    /// Frees the vertex buffer with the specified ID
    ///
    fn free_vertex_buffer(&mut self, VertexBufferId(id): VertexBufferId) {
        self.vertex_buffers_2d[id] = None;
    }

    ///
    /// Creates a new BGRA texture
    ///tz
    fn create_bgra_texture(&mut self, TextureId(id): TextureId, width: usize, height: usize) {
        // Extend the texture list if needed
        if self.bgra_textures.len() <= id {
            self.bgra_textures.extend((self.bgra_textures.len()..(id+1))
                .into_iter().map(|_| None));
        }

        // Create a new texture
        self.bgra_textures[id] = None;
        self.rgba_textures[id] = None;

        let new_texture = self.factory.create_texture(
            texture::Kind::D2(width as u16, height as u16, texture::AaMode::Multi(4)),
            1,
            memory::Bind::SHADER_RESOURCE,
            memory::Usage::Upload,
            None).unwrap();
        self.bgra_textures[id] = Some(new_texture);
    }

    ///
    /// Releases an existing render target
    ///
    fn free_texture(&mut self, TextureId(texture_id): TextureId) {
        self.bgra_textures[texture_id] = None;
        self.rgba_textures[texture_id] = None;
    }

    ///
    /// Creates a new render target
    ///
    fn create_render_target(&mut self, RenderTargetId(render_id): RenderTargetId, TextureId(texture_id): TextureId, width: usize, height: usize, render_type: RenderTargetType) {
        // We store the 'main' render target in ID 0 so we shift all IDs up by 1 to prevent overwriting it
        let render_id = render_id + 1;

        // Extend the texture list if needed
        if self.bgra_textures.len() <= texture_id {
            self.bgra_textures.extend((self.bgra_textures.len()..(texture_id+1))
                .into_iter().map(|_| None));
        }

        // Extend the render target list if needed
        if self.render_targets.len() <= render_id {
            self.render_targets.extend((self.render_targets.len()..(render_id+1))
                .into_iter().map(|_| None));
        }

        // Clear out any existing texture/render target
        self.bgra_textures[texture_id] = None;
        self.render_targets[render_id] = None;

        let aa_mode = match render_type {
            RenderTargetType::Standard      => texture::AaMode::Single,
            RenderTargetType::Multisampled  => texture::AaMode::Multi(4)
        };

        // Create the backing texture
        let new_texture                 = self.factory.create_texture(
            texture::Kind::D2(width as u16, height as u16, aa_mode),
            1,
            memory::Bind::SHADER_RESOURCE,
            memory::Usage::Upload,
            None).unwrap();

        // Create the render target
        let new_render_target           = self.factory.view_texture_as_render_target(&new_texture, 0, None).unwrap();
        let new_render_target           = RenderTarget { render_target: new_render_target };

        // Store the resources
        self.bgra_textures[texture_id]  = Some(new_texture);
        self.render_targets[render_id]  = Some(new_render_target);
    }

    ///
    /// Chooses which buffer rendering instructions will be sent to
    ///
    fn select_render_target(&mut self, RenderTargetId(render_id): RenderTargetId) {
        // We store the 'main' render target in ID 0 so we shift all IDs up by 1 to prevent overwriting it
        let render_id = render_id + 1;

        self.active_render_target = render_id;
    }

    ///
    /// Sends rendering instructions to the primary frame buffer for display
    ///
    fn select_main_frame_buffer(&mut self) {
        self.active_render_target = 0;
    }

    ///
    /// Releases an existing render target
    ///
    fn free_render_target(&mut self, RenderTargetId(render_id): RenderTargetId) {
        self.render_targets[render_id] = None;
    }

    ///
    /// Flushes all changes to the device
    ///
    pub fn flush(&mut self) {
        self.encoder.flush(&mut self.device);
    }
}
