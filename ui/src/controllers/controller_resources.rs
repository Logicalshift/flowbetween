use crate::image::*;
use crate::binding_canvas::*;
use crate::resource_manager::*;

use std::sync::*;

///
/// A collection of all the resource managers needed to implement a controller
///
#[derive(Clone)]
pub struct ControllerResources {
    /// The images for the controller
    images: Arc<ResourceManager<Image>>,

    /// The canvases for the controller
    canvases: Arc<ResourceManager<BindingCanvas>>
}

impl ControllerResources {
    ///
    /// Creates a new controller resources structure
    ///
    pub fn new() -> ControllerResources {
        ControllerResources {
            images:     Arc::new(ResourceManager::new()),
            canvases:   Arc::new(ResourceManager::new())
        }
    }

    ///
    /// Returns the image resource manager used for this controller
    ///
    pub fn images(&self) -> &Arc<ResourceManager<Image>> {
        &self.images
    }

    ///
    /// Returns the canvas resource manager used for this controller
    ///
    pub fn canvases(&self) -> &Arc<ResourceManager<BindingCanvas>> {
        &self.canvases
    }
}