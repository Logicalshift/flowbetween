use super::layer::*;

///
/// Represents an animation
///
pub trait Animation {
    ///
    /// Retrieves the size of this animation
    ///
    fn size() -> (f32, f32);

    ///
    /// Retrieves the layers for this animation
    ///
    fn layers(&self) -> Box<Iterator<Item = Layer>>;
}
