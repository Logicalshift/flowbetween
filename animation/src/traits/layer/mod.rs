mod paint;

pub use self::paint::*;

use super::edit::*;
use super::frame::*;

use canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// A layer represents a renderable plane in an animation
///
pub trait Layer : 
    Send+Sync {
    ///
    /// The ID associated with this layer
    /// 
    fn id(&self) -> u64;

    ///
    /// The types of edit that are supported by this layer
    /// 
    fn supported_edit_types(&self) -> Vec<LayerEditType>;

    ///
    /// Retrieves the definition of this layer as a paint layer
    /// 
    fn as_paint_layer<'a>(&'a self) -> Option<&'a PaintLayer>;

    ///
    /// Renders the result of the specified set of actions to the given graphics primitives
    /// 
    fn draw_pending_actions(&self, gc: &mut GraphicsPrimitives, pending: &PendingEditLog<LayerEdit>);    

    ///
    /// Retrieves a frame from this layer with the specified parameters
    ///
    fn get_frame_at_time(&self, time_index: Duration) -> Arc<Frame>;

    ///
    /// Retrieves the times where key frames exist
    ///
    fn get_key_frames(&self) -> Box<Iterator<Item=Duration>>;
}
