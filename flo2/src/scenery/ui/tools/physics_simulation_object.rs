use super::physics_simulation_joints::*;
use crate::scenery::ui::ui_path::*;

use flo_binding::*;
use rapier2d::prelude::*;

use uuid::*;
use ::serde::*;
use smallvec::*;

///
/// Identifier of an object in a physics simulation
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimObjectId(Uuid);

impl SimObjectId {
    ///
    /// Creates a unique new simulation object ID
    ///
    pub fn new() -> Self {
        SimObjectId(Uuid::new_v4())
    }
}

///
/// Shapes permitted by a simulation object
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimObjectType {
    /// Object that does not move
    Static,

    /// Object that moves using the simulation (when MoveTo is called, we move this object by applying a force to it rather than teleporting it)
    Dynamic,

    /// Object that can have its coordiantes set immediately
    Kinematic,
}

///
/// Data stored within the physics simulation for an object
///
pub (super) struct SimObject {
    /// The object ID used to refer to this object outside of the simulation
    pub (super) _object_id: SimObjectId,

    /// Handle of the rigid body attached to this object
    pub (super) rigid_body_handle: RigidBodyHandle,

    /// The collider that is associated with this object
    pub (super) collider_handle: Option<ColliderHandle>,

    /// The type of rigid body that this object is being simulated as
    pub (super) body_type: SimObjectType,

    /// The spring joint used to anchor this object to its preferred position
    pub (super) anchor_joint: Option<(RigidBodyHandle, ImpulseJointHandle)>,

    /// The other joints that are attached to this object
    pub (super) joints: SmallVec<[SimJointId; 1]>,

    /// The impulse binding for this object
    pub (super) impulse: Option<BindRef<UiPoint>>,

    /// The objects that this object will not collide with
    pub (super) collision_exclusions: Option<BindRef<Vec<SimObjectId>>>,
}

impl SimObject {
    ///
    /// Creates the standard kinematic position based object
    ///
    pub fn kinematic_position_based(object_id: SimObjectId, rigid_body_handle: RigidBodyHandle) -> Self {
        Self {
            _object_id:             object_id,
            rigid_body_handle:      rigid_body_handle,
            collider_handle:        None,
            body_type:              SimObjectType::Kinematic,
            anchor_joint:           None,
            joints:                 smallvec![],
            impulse:                None,
            collision_exclusions:   None,
        }
    }

    ///
    /// Updates the body type of this object
    ///
    #[inline]
    pub fn set_body_type(&mut self, new_type: SimObjectType) {
        self.body_type = new_type;
    }
}
