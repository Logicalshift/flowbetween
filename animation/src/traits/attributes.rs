use super::super::inmemory::*;

use ui::canvas::*;

use std::any::*;

///
/// Implementation of an attribute attached to an animation item
/// 
pub trait AnimationAttribute : Any {
    ///
    /// Renders the contents of this attribute to the specified animation context
    ///
    fn render(&self, context: &mut GraphicsContext);
}

///
/// Anything with attributes can implement the HasAttributes trait
///
pub trait HasAttributes {
    ///
    /// Retrieves the attributes attached to this item
    ///
    fn attributes<'a>(&'a self) -> Box<'a+Iterator<Item = &'a AnimationAttribute>>;
}

impl HasAttributes for () {
    fn attributes<'a>(&'a self) -> Box<'a+Iterator<Item = &'a AnimationAttribute>> {
        Box::new(EmptyIterator::new())
    }
}
