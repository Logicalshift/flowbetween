use super::frame::*;

///
/// A layer represents a renderable plane in an animation
///
pub trait Layer {
    ///
    /// The ID associated with this layer
    /// 
    fn id(&self) -> u64;

    ///
    /// Retrieves a frame from this layer with the specified parameters
    ///
    fn get_frame_at_time(&self, time_index_nanos: u64);

    ///
    /// Retrieves the key frames in this layer
    ///
    fn get_key_frames<'a>(&'a self) -> Box<'a+Iterator<Item = &'a Frame>>;
}
