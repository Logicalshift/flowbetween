use super::edit::*;
use super::layer::*;
use super::editable::*;

use std::sync::*;

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
    /// Retrieves the layer with the specified ID from this animation
    /// 
    fn get_layer_with_id(&self, layer_id: u64) -> Option<Arc<Layer>>;

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
    fn edit_layer<'a>(&'a self) -> Editor<'a, PendingEditLog<LayerEdit>>;
}
