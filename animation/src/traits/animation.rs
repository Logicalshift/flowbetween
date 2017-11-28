use super::layer::*;
use super::editable::*;
use super::attributes::*;
use super::super::inmemory::*;

///
/// Represents an animation
///
pub trait Animation : Editable<AnimationLayers>+Editable<AnimationSize>+HasAttributes+Send+Sync {
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
/// Represents the size properties of an animation
/// 
pub trait AnimationSize {
    ///
    /// Retrieves the frame size of this animation
    /// 
    fn size(&self) -> (f64, f64);

    ///
    /// Updates the frame size of this animation
    ///
    fn set_size(&self, (f64, f64));
}

///
/// Represents the layers associated with an animation
/// 
pub trait AnimationLayers {
    ///
    /// Retrieves the layers for this animation
    ///
    fn layers<'a>(&'a self) -> Box<'a+Iterator<Item = &'a Layer>>;

    ///
    /// Removes the layer with the specified ID
    /// 
    fn remove_layer(&mut self, layer_id: u64);

    ///
    /// Adds a new layer to this object
    /// 
    fn add_new_layer<'a>(&'a mut self) -> &'a Layer;
}

impl Animation for () {
    fn size(&self) -> (f64, f64) {
        (1980.0, 1080.0)
    }

    fn layers<'a>(&'a self) -> Box<Iterator<Item = &'a Layer>> {
        Box::new(EmptyIterator::new())
    }
}
