use super::layer::*;

///
/// Represents an animation
///
pub trait Animation {
    ///
    /// Retrieves the layers for this animation
    ///
    fn layers(&self) -> Box<Iterator<Item = Layer>>;
}
