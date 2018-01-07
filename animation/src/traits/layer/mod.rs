mod vector;

pub use self::vector::*;

use super::edit::*;
use super::frame::*;
use super::editable::*;

use std::sync::*;
use std::time::Duration;

///
/// A layer represents a renderable plane in an animation
///
pub trait Layer : 
    Send+Sync {
    ///
    /// The ID associated with this layer
    /// 
    fn id(&self) -> u64;

    ///
    /// The types of edit that are supported by this layer
    /// 
    fn supported_edit_types(&self) -> Vec<LayerEditType>;

    ///
    /// Retrieves a frame from this layer with the specified parameters
    ///
    fn get_frame_at_time(&self, time_index: Duration) -> Arc<Frame>;

    ///
    /// Retrieves the times where key frames exist
    ///
    fn get_key_frames(&self) -> Box<Iterator<Item=Duration>>;

    ///
    /// Adds a new key frame at the specified time
    /// 
    fn add_key_frame(&mut self, when: Duration);

    ///
    /// Retrieves the definition of this layer as a vector layer
    /// 
    fn as_vector_layer<'a>(&'a self) -> Option<Reader<'a, VectorLayer>>;

    ///
    /// Retrieves an editor for the vector layer
    /// 
    fn edit_vectors<'a>(&'a mut self) -> Option<Editor<'a, VectorLayer>>;
}
