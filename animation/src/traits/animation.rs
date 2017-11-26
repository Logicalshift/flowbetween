use super::layer::*;
use super::attributes::*;
use super::super::inmemory::*;

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
    fn layers<'a>(&'a self) -> Box<'a+Iterator<Item = &'a Layer>>;
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
        Box::new(EmptyIterator::new())
    }
}
