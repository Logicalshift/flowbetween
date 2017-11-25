use super::layer::*;
use super::attributes::*;

///
/// Represents an animation
///
pub trait Animation : HasAttributes {
    ///
    /// Retrieves the layers for this animation
    ///
    fn layers<'a>(&'a self) -> Box<Iterator<Item = &'a Layer>>;
}
