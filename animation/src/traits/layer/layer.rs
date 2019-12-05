use super::vector::*;
use super::super::edit::*;
use super::super::frame::*;
use super::super::cache::*;

use std::u32;
use std::sync::*;
use std::time::Duration;
use std::ops::{Range, Deref};

///
/// A layer represents a renderable plane in an animation
///
pub trait Layer : Send+Sync {
    ///
    /// The ID associated with this layer
    ///
    fn id(&self) -> u64;

    ///
    /// Retrieves the name associated with this layer (or none if no name has been assigned yet)
    ///
    fn name(&self) -> Option<String>;

    ///
    /// The types of edit that are supported by this layer
    ///
    fn supported_edit_types(&self) -> Vec<LayerEditType>;

    ///
    /// Retrieves a frame from this layer with the specified parameters
    ///
    fn get_frame_at_time(&self, time_index: Duration) -> Arc<dyn Frame>;

    ///
    /// Retrieves the times where key frames exist
    ///
    fn get_key_frames(&self) -> Box<dyn Iterator<Item=Duration>> { self.get_key_frames_during_time(Duration::from_millis(0)..Duration::from_secs(u32::MAX as u64)) }

    ///
    /// Retrieves the times where key frames exist during a specified time range
    ///
    fn get_key_frames_during_time(&self, when: Range<Duration>) -> Box<dyn Iterator<Item=Duration>>;

    ///
    /// Retrieves the previous and next keyframes from a particular point in time
    ///
    /// (If there's a keyframe at this point in time, it is not returned)
    ///
    fn previous_and_next_key_frame(&self, when: Duration) -> (Option<Duration>, Option<Duration>);

    ///
    /// Retrieves the definition of this layer as a vector layer
    ///
    fn as_vector_layer<'a>(&'a self) -> Option<Box<dyn 'a+Deref<Target=dyn 'a+VectorLayer>>>;

    ///
    /// Retrieves the canvas cache at the specified time
    ///
    fn get_canvas_cache_at_time(&self, time_index: Duration) -> Arc<dyn CanvasCache>;
}
