use crate::scenery::ui::ui_path::*;

use uuid::*;
use ::serde::*;

///
/// Indentifier of a joint between objects in a physics simulation
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimJointId(Uuid);

impl SimJointId {
    ///
    /// Creates a unique new simulation joint ID
    ///
    pub fn new() -> Self {
        SimJointId(Uuid::new_v4())
    }
}

///
/// Basic definition of a joint
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimJoint {
    /// A fixed joint (stops two objects from moving apart)
    FixedJoint,

    /// A spring joint (pulls an object towards a fixed distance away)
    SpringJoint { rest_length: f64, stiffness: f64, damping: f64 },

    /// A rope joint (ensures an object can't get more than a certain distance away)
    RopeJoint { max_dist: f64 },
}

///
/// Extra properties that can be applied to a joint
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimJointSide {
    /// The first object that the joint is for
    First,

    /// The second object that the joint is for
    Second,
}

///
/// Extra properties that can be applied to a joint
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimJointProperty {
    /// Whether or not the objects at both ends of the joint can collide with each other
    ContactsEnabled(bool),

    /// Offset from the center where the joint connects with the object
    LocalAnchor(SimJointSide, UiPoint),
}
