use super::edit::*;
use super::layer::*;

use futures::*;

use std::time::Duration;

///
/// Represents an animation
///
pub trait Animation : 
    Send+Sync {
    ///
    /// Retrieves the frame size of this animation
    /// 
    fn size(&self) -> (f64, f64);

    ///
    /// Retrieves the length of this animation
    /// 
    fn duration(&self) -> Duration;

    ///
    /// Retrieves the duration of a single frame
    /// 
    fn frame_length(&self) -> Duration;

    ///
    /// Retrieves the IDs of the layers in this object
    /// 
    fn get_layer_ids(&self) -> Vec<u64>;

    ///
    /// Retrieves the layer with the specified ID from this animation
    /// 
    fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<&'a Layer>;

    ///
    /// Retrieves the log for this animation
    /// 
    fn get_log<'a>(&'a self) -> Box<'a+Stream<Item=AnimationEdit, Error=()>>;
}

///
/// Represents something that can edit an animation
/// 
pub trait EditableAnimation {
    ///
    /// Retrieves a sink that can be used to send edits for this animation
    /// 
    /// Edits are supplied as groups (stored in a vec) so that it's possible to ensure that
    /// a set of related edits are performed atomically
    /// 
    fn edit<'a>(&'a self) -> Box<'a+Sink<Item=Vec<AnimationEdit>, Error=()>>;
}
