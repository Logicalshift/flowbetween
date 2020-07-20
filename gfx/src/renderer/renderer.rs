use crate::action::*;

use gfx;

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
    encoder: gfx::Encoder<Device::Resources, Device::CommandBuffer>
}

impl<Device, Factory> Renderer<Device, Factory>
where   Device:     gfx::Device,
        Factory:    gfx::Factory<Device::Resources> {
    ///
    /// Creates a new renderer that will render to the specified device and factory
    ///
    pub fn new(device: Device, factory: Factory, encoder: gfx::Encoder<Device::Resources, Device::CommandBuffer>) -> Renderer<Device, Factory> {
        Renderer {
            device:     device,
            factory:    factory,
            encoder:    encoder
        }
    }

    ///
    /// Performs rendering of the specified actions to this device target
    ///
    pub fn render<Actions: IntoIterator<Item=GfxAction>>(&mut self, actions: Actions) {
        for action in actions {
            use self::GfxAction::*;

            match action {
                CreateVertex2DBuffer(id, vertices)                                      => { }
                FreeVertexBuffer(id)                                                    => { }
                CreateRenderTarget(render_id, texture_id, width, height, render_type)   => { }
                FreeRenderTarget(render_id)                                             => { }
                SelectRenderTarget(render_id)                                           => { }
                RenderToFrameBuffer                                                     => { }
                ShowFrameBuffer                                                         => { }
                CreateTextureRgba(texture_id, width, height)                            => { }
                LoadTextureData(texture_id, offset, data)                               => { }
                Clear(color)                                                            => { self.clear(color); }
            }
        }
    }

    ///
    /// Clears the current render target
    ///
    fn clear(&mut self, color: Rgba8) {
        let Rgba8([r, g, b, a]) = color;
        let (r, g, b, a)        = ((r as f32)/255.0, (g as f32)/255.0, (b as f32)/255.0, (a as f32)/255.0);

        // self.encoder.clear([r, g, b, a);
    }
}
