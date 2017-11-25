use super::layer::*;
use super::attributes::*;

///
/// Represents an animation
///
pub trait Animation : HasAttributes {
    ///
    /// Retrieves the size of this animation
    ///
    fn size() -> (f32, f32);

    ///
    /// Retrieves the layers for this animation
    ///
    fn layers(&self) -> Box<Iterator<Item = Layer>>;
}
