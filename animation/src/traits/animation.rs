use super::edit::*;
use super::layer::*;
use super::editable::*;

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
    fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<Reader<'a, Layer>>;

    ///
    /// Retrieves the log for this animation
    /// 
    fn get_log<'a>(&'a self) -> Reader<'a, EditLog<AnimationEdit>>;

    ///
    /// Retrieves an edit log that can be used to alter this animation
    /// 
    fn edit<'a>(&'a self) -> Editor<'a, PendingEditLog<AnimationEdit>>;

    ///
    /// Retrieves an edit log that can be used to edit a layer in this animation
    /// 
    fn edit_layer<'a>(&'a self, layer_id: u64) -> Editor<'a, PendingEditLog<LayerEdit>>;
}

///
/// Trait implemented by objects that support editing the data associated with an animation
/// 
/// Normally edits are made by sending them via the `edit()` method in 
/// `Animation`. This used to edit the actual data structure associated
/// with an animation.
/// 
pub trait MutableAnimation :
    Send {
    ///
    /// Sets the canvas size of this animation
    ///
    fn set_size(&mut self, size: (f64, f64));

    ///
    /// Creates a new layer with a particular ID
    /// 
    /// Has no effect if the layer ID is already in use
    /// 
    fn add_layer(&mut self, new_layer_id: u64);

    ///
    /// Removes the layer with the specified ID
    /// 
    fn remove_layer(&mut self, old_layer_id: u64);

    ///
    /// Opens a particular layer for editing
    /// 
    fn edit_layer<'a>(&'a mut self, layer_id: u64) -> Option<Editor<'a, Layer>>;
}
