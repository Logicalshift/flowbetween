use super::edit::*;
use super::layer::*;
use super::animation_motion::*;

use futures::*;

use std::time::Duration;
use std::sync::*;
use std::ops::Range;

///
/// Represents an animation
///
pub trait Animation : Send+Sync {
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
    fn get_layer_with_id(&self, layer_id: u64) -> Option<Arc<dyn Layer>>;

    ///
    /// Retrieves the total number of items that have been performed on this animation
    /// 
    fn get_num_edits(&self) -> usize;

    ///
    /// Reads from the edit log for this animation
    /// 
    fn read_edit_log<'a>(&'a self, range: Range<usize>) -> Box<dyn 'a+Stream<Item=AnimationEdit, Error=()>>;

    ///
    /// Supplies a reference which can be used to find the motions associated with this animation
    /// 
    fn motion<'a>(&'a self) -> &'a dyn AnimationMotion;
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
    fn edit(&self) -> Box<dyn Sink<SinkItem=Vec<AnimationEdit>, SinkError=()>+Send>;
}
