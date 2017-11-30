mod paint;
mod keyframes;

pub use self::paint::*;
pub use self::keyframes::*;

use super::frame::*;
use super::editable::*;
use std::time::Duration;

///
/// A layer represents a renderable plane in an animation
///
pub trait Layer : 
    Editable<PaintLayer>+
    Editable<KeyFrameLayer>+
    Send+Sync {
    ///
    /// The ID associated with this layer
    /// 
    fn id(&self) -> u64;

    ///
    /// Retrieves a frame from this layer with the specified parameters
    ///
    fn get_frame_at_time<'a>(&self, time_index: Duration) -> &'a Frame;

    ///
    /// Retrieves the times where key frames exist
    ///
    fn get_key_frames(&self) -> Box<Iterator<Item=Duration>>;
}
