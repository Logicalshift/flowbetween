///
/// Indicates a part of the main canvas that can have become invalid
///
#[derive(Clone, Copy, Debug)]
pub enum CanvasInvalidation {
    /// The whole canvas needs refreshing
    WholeCanvas,

    /// A layer has become invalid (identified by the FrameLayerModel layer ID)
    Layer(u64)
}
