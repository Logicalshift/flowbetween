use super::edit::*;
use super::motion::*;

use futures::*;

use std::time::Duration;
use std::ops::Range;

///
/// Supplies the motion elements for an animation
/// 
pub trait AnimationMotion {
    ///
    /// Assigns a new unique ID for creating a new motion
    /// 
    /// (This ID will not have been used so far and will not be used again)
    /// 
    fn assign_motion_id(&self) -> ElementId;

    ///
    /// Retrieves a stream containing all of the motions in a particular time range
    /// 
    fn get_motion_ids(&self, when: Range<Duration>) -> Box<Stream<Item=ElementId, Error=()>>;

    ///
    /// Retrieves the IDs of the motions attached to a particular element
    /// 
    fn get_motions_for_element(&self, element_id: ElementId) -> Vec<ElementId>;

    ///
    /// Retrieves the motion with the specified ID
    /// 
    fn get_motion(&self, motion_id: ElementId) -> Option<Motion>;
}