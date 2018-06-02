use super::edit::*;
use super::layer::*;
use super::motion::*;

use futures::*;

use std::time::Duration;
use std::ops::{Range, Deref};

///
/// Supplies the motion elements for an animation
/// 
pub trait AnimationMotion {
    ///
    /// Retrieves a stream containing all of the motions in a particular time range
    /// 
    fn get_motion_ids(&self, when: Range<Duration>) -> Box<Stream<Item=ElementId, Error=()>>;

    ///
    /// Retrieves the motion with the specified ID
    /// 
    fn get_motion(&self, motion_id: ElementId) -> Option<Motion>;
}