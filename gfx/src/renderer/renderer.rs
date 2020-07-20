use crate::action::*;

use gfx;

///
/// Renders GFX actions to a GFX device
///
pub struct Renderer<Device, Factory>
where   Device:     gfx::Device,
        Factory:    gfx::Factory<Device::Resources> {
    device:  Device,
    factory: Factory
}

impl<Device, Factory> Renderer<Device, Factory>
where   Device:     gfx::Device,
        Factory:    gfx::Factory<Device::Resources> {
    ///
    /// Creates a new renderer that will render to the specified device and factory
    ///
    pub fn new(device: Device, factory: Factory) -> Renderer<Device, Factory> {
        Renderer {
            device:     device,
            factory:    factory
        }
    }
}
