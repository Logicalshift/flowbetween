///
/// Represents a parameter that is passed into a frame for rendering purposes
///
pub enum FrameParameter {
    /// The time index of this frame (number of nanoseconds since the start of the animation)
    TimeIndex(u64)
}
