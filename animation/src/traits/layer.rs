use super::frame_parameter::*;
use super::frame::*;

///
/// A layer represents a renderable plane in an animation
///
pub trait Layer {
    ///
    /// Retrieves a frame from this layer with the specified parameters
    ///
    fn get_frame(&self, Box<Iterator<Item = FrameParameter>>) -> Box<Frame>;
}
