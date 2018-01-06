use super::edit::*;
use super::layer::*;
use super::editable::*;

use std::sync::*;

///
/// Represents an animation
///
pub trait Animation : 
    Editable<AnimationLayers>+
    Editable<EditLog<AnimationEdit>>+
    Editable<MutableEditLog<AnimationEdit>>+
    Send+Sync {
    ///
    /// Retrieves the frame size of this animation
    /// 
    fn size(&self) -> (f64, f64);
}

///
/// Represents the layers associated with an animation
/// 
pub trait AnimationLayers {
    ///
    /// Retrieves the layers for this animation
    ///
    fn layers<'a>(&'a self) -> Box<'a+Iterator<Item = &'a Arc<Layer>>>;
}
