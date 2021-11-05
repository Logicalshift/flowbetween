use crate::image::*;
use crate::binding_canvas::*;
use crate::resource_manager::*;
use crate::dynamic_viewmodel::*;

use std::sync::*;

///
/// A stream controller implements the controller trait and runs the controller by dispatching and receiving messages
/// on a stream. This allows for writing controllers that run as streams, and also removes most of the boilerplate
/// around setting up resource managers and view models.
///
/// A future-based runtime like this also makes it easier to update the controller in the background and manage different
/// states (say when tracking mouse drags and drawing actions)
///
pub struct StreamController<TFuture> {
    /// The runtime for the controller
    runtime: TFuture,

    /// The viewmodel for this controller
    viewmodel: DynamicViewModel,

    /// The canvases for this stream controller
    canvases: Arc<ResourceManager<BindingCanvas>>,

    /// The images for this stream controller
    images: Arc<ResourceManager<Image>>
}
