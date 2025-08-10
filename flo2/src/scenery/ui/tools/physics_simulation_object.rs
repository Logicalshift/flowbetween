use super::physics_simulation_joints::*;
use crate::scenery::ui::ui_path::*;

use flo_binding::*;
use flo_curves::*;
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
/// Bindings for a SimObject that specify its actual state within the simulation
///
/// Not to be confused with the bindings that are set externally and specify the object's desired state. These bindings
/// are updated when the simulation runs to set the state of the object
///
pub (super) struct SimObjectStateBindings {
    pub (super) position:         Option<Binding<UiPoint>>,
    pub (super) angle:            Option<Binding<f64>>,
    pub (super) velocity:         Option<Binding<UiPoint>>,
    pub (super) angular_velocity: Option<Binding<f64>>,
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

    /// True if this object is new (hasn't been run through the simulation yet)
    pub (super) is_new: bool,

    /// The type of rigid body that this object is being simulated as
    pub (super) body_type: SimObjectType,

    /// The spring joint used to anchor this object to its preferred position
    pub (super) anchor_joint: Option<(RigidBodyHandle, ImpulseJointHandle)>,

    /// The other joints that are attached to this object
    pub (super) joints: SmallVec<[SimJointId; 1]>,

    /// The binding of the position of this object
    pub (super) position: Option<BindRef<UiPoint>>,

    /// The impulse binding for this object
    pub (super) impulse: Option<BindRef<UiPoint>>,

    /// The objects that this object will not collide with
    pub (super) collision_exclusions: Option<BindRef<Vec<SimObjectId>>>,

    /// The state of this object within the simulation
    pub (super) state_bindings: SimObjectStateBindings,
}

impl SimObject {
    ///
    /// Creates the standard kinematic position based object
    ///
    pub fn kinematic_position_based(object_id: SimObjectId, rigid_body_handle: RigidBodyHandle) -> Self {
        let state_bindings = SimObjectStateBindings { 
            position:           None, 
            angle:              None, 
            velocity:           None, 
            angular_velocity:   None,
        };

        Self {
            _object_id:             object_id,
            rigid_body_handle:      rigid_body_handle,
            collider_handle:        None,
            is_new:                 true,
            body_type:              SimObjectType::Kinematic,
            anchor_joint:           None,
            joints:                 smallvec![],
            position:               None,
            impulse:                None,
            collision_exclusions:   None,
            state_bindings:         state_bindings,
        }
    }

    ///
    /// Updates the body type of this object
    ///
    #[inline]
    pub fn set_body_type(&mut self, new_type: SimObjectType) {
        self.body_type = new_type;
    }

    ///
    /// Updates this object's position within the simulation to the specified value
    ///
    pub fn update_position(&mut self, new_position: UiPoint, rigid_body_set: &mut RigidBodySet, impulse_joint_set: &mut ImpulseJointSet) {
        let Some(rigid_body) = rigid_body_set.get_mut(self.rigid_body_handle) else { return; };

        if self.is_new {
            // Set the position immediately if the body is new
            rigid_body.set_position(Isometry::new(vector![new_position.x() as _, new_position.y() as _], 0.0), true);
        } else {
            // Action for setting the position depends on the type of the object
            match self.body_type {
                SimObjectType::Static       => { rigid_body.set_position(Isometry::new(vector![new_position.x() as _, new_position.y() as _], 0.0), true); }
                SimObjectType::Kinematic    => { rigid_body.set_next_kinematic_position(Isometry::new(vector![new_position.x() as _, new_position.y() as _], 0.0)); rigid_body.wake_up(false); }
                SimObjectType::Dynamic      => { /* We set up a spring to pull the object into position */ }
            }
        }

        let anchor_handle = if let Some((anchor_handle, _)) = self.anchor_joint {
            // Use the existing anchor
            anchor_handle
        } else {
            // Create an anchor for this point
            let anchor          = RigidBodyBuilder::fixed().position(Isometry::new(vector![new_position.x() as _, new_position.y() as _], 0.0)).build();
            let anchor_handle   = rigid_body_set.insert(anchor);

            // Create a spring to attach to the anchor
            let spring          = SpringJointBuilder::new(0.0, 100.0, 10.0).spring_model(MotorModel::AccelerationBased).contacts_enabled(false).build();
            let spring_handle   = impulse_joint_set.insert(anchor_handle, self.rigid_body_handle, spring, true);

            self.anchor_joint   = Some((anchor_handle, spring_handle));

            anchor_handle
        };

        // Set the position of the anchor (where the spring will draw this object to, when it's dynamic)
        if let Some(anchor) = rigid_body_set.get_mut(anchor_handle) {
            anchor.set_position(Isometry::new(vector![new_position.x() as _, new_position.y() as _], 0.0), true);
        }
    }
}
