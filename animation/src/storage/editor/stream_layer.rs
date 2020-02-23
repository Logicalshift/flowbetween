use super::stream_animation_core::*;
use super::super::layer_properties::*;
use super::super::super::traits::*;

use ::desync::*;

use std::sync::*;
use std::time::{Duration};
use std::ops::{Range, Deref};

///
/// A layer from a stream animation
///
pub struct StreamLayer {
    /// The core, where the actual work is done
    core: Arc<Desync<StreamAnimationCore>>,

    /// The ID of the layer that this should fetch
    layer_id: u64,

    /// The properties for this layer
    properties: LayerProperties
}

impl StreamLayer {
    ///
    /// Creates a new stream layer from a core, a layer ID and some layer properties
    ///
    pub (super) fn new(core: Arc<Desync<StreamAnimationCore>>, layer_id: u64, properties: LayerProperties) -> StreamLayer {
        StreamLayer {
            core:           core,
            layer_id:       layer_id,
            properties:     properties
        }
    }
}

impl Layer for StreamLayer {
    ///
    /// The ID associated with this layer
    ///
    fn id(&self) -> u64 {
        self.layer_id
    }

    ///
    /// Retrieves the name associated with this layer (or none if no name has been assigned yet)
    ///
    fn name(&self) -> Option<String> {
        Some(self.properties.name.clone())
    }

    ///
    /// The types of edit that are supported by this layer
    ///
    fn supported_edit_types(&self) -> Vec<LayerEditType> {
        vec![
            LayerEditType::Vector
        ]
    }

    ///
    /// Retrieves a frame from this layer with the specified parameters
    ///
    fn get_frame_at_time(&self, time_index: Duration) -> Arc<dyn Frame> {
        unimplemented!("get_frame_at_time")
    }

    ///
    /// Retrieves the times where key frames exist during a specified time range
    ///
    fn get_key_frames_during_time(&self, when: Range<Duration>) -> Box<dyn Iterator<Item=Duration>> {
        unimplemented!("get_key_frames_during_time")
    }

    ///
    /// Retrieves the previous and next keyframes from a particular point in time
    ///
    /// (If there's a keyframe at this point in time, it is not returned)
    ///
    fn previous_and_next_key_frame(&self, when: Duration) -> (Option<Duration>, Option<Duration>) {
        unimplemented!("previous_and_next_key_frame")
    }

    ///
    /// Retrieves the definition of this layer as a vector layer
    ///
    fn as_vector_layer<'a>(&'a self) -> Option<Box<dyn 'a+Deref<Target=dyn 'a+VectorLayer>>> {
        unimplemented!("as_vector_layer")
    }

    ///
    /// Retrieves the canvas cache at the specified time
    ///
    fn get_canvas_cache_at_time(&self, time_index: Duration) -> Arc<dyn CanvasCache> {
        unimplemented!("get_canvas_cache_at_time")
    }
}
