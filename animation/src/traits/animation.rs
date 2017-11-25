use super::layer::*;
use super::attributes::*;

///
/// Represents an animation
///
pub trait Animation : HasAttributes+Send+Sync {
    ///
    /// Returns the dimensions of this animation
    /// 
    fn size(&self) -> (f64, f64);

    ///
    /// Retrieves the layers for this animation
    ///
    fn layers<'a>(&'a self) -> Box<Iterator<Item = &'a Layer>>;
}

///
/// Represents an animation that can be edited
/// 
pub trait EditableAnimation : Animation {

}

impl EditableAnimation for () {
}

impl Animation for () {
    fn size(&self) -> (f64, f64) {
        (1980.0, 1080.0)
    }

    fn layers<'a>(&'a self) -> Box<Iterator<Item = &'a Layer>> {
        // TODO: return an empty iterator
        unimplemented!()
    }
}
