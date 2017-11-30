use super::frame::*;
use super::frame_parameter::*;

use std::any::*;

///
/// A layer represents a renderable plane in an animation
///
pub trait Layer : Any {
    ///
    /// Retrieves a frame from this layer with the specified parameters
    ///
    fn get_frame(&self, parameters: &mut Iterator<Item = FrameParameter>) -> Box<Frame>;

    ///
    /// Retrieves the key frames in this layer
    ///
    fn get_key_frames<'a>(&'a self) -> Box<'a+Iterator<Item = &'a Frame>>;
}
