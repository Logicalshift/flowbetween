use std::time::Duration;

///
/// Editing trait provided by layers that can have key frames. Key frames reset the
/// state of the layer.
/// 
pub trait KeyFrameLayer {
    ///
    /// Adds a new key frame at the specified time offset from the start of the
    /// animation.
    /// 
    fn add_key_frame(&mut self, time_offset: Duration);
}
