use super::physics_simulation_joints::*;
use crate::scenery::ui::ui_path::*;

use flo_binding::*;
use flo_binding::binding_context::*;
use flo_curves::*;
use rapier2d::prelude::*;

use uuid::*;
use ::serde::*;
use smallvec::*;

use std::collections::{HashMap};
use std::sync::*;

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
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SimObjectType {
    /// Object that does not move
    Static,

    /// Object that moves using the simulation (when MoveTo is called, we move this object by applying a force to it rather than teleporting it)
    Dynamic,

    /// Object that can have its coordiantes set immediately
    Kinematic,
}

///
/// Represents a collection of objects
///
pub (super) struct SimObjectCollection {
    /// The objects in this collection
    bodies: HashMap<SimObjectId, SimObject>,

    /// The object ID for each handle
    id_for_handle: HashMap<RigidBodyHandle, SimObjectId>,
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
    pub (super) object_id: SimObjectId,

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

    /// If we're tracking when the bindings of this object are changed, this is the releasable that represents that lifetime
    pub (super) bindings_changed: Option<Mutex<Box<dyn Send + Releasable>>>,

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
            object_id:              object_id,
            rigid_body_handle:      rigid_body_handle,
            collider_handle:        None,
            is_new:                 true,
            body_type:              SimObjectType::Kinematic,
            anchor_joint:           None,
            joints:                 smallvec![],
            bindings_changed:       None,
            position:               None,
            impulse:                None,
            collision_exclusions:   None,
            state_bindings:         state_bindings,
        }
    }

    ///
    /// Updates the body type of this object
    ///
    pub fn set_body_type(&mut self, new_type: SimObjectType, rigid_body_set: &mut RigidBodySet, collider_set: &mut ColliderSet) {
        self.body_type = new_type;

        let Some(rigid_body) = rigid_body_set.get_mut(self.rigid_body_handle) else { return; };
        match new_type {
            SimObjectType::Static    => { rigid_body.set_body_type(RigidBodyType::Fixed, true); }
            SimObjectType::Dynamic   => { rigid_body.set_body_type(RigidBodyType::Dynamic, true); }
            SimObjectType::Kinematic => { rigid_body.set_body_type(RigidBodyType::KinematicPositionBased, true); }
        }

        if let Some(collider) = self.collider_handle.and_then(|handle| collider_set.get_mut(handle)) {
            match new_type {
                SimObjectType::Static    => { collider.set_active_hooks(ActiveHooks::default()); }
                SimObjectType::Dynamic   => { collider.set_active_hooks(ActiveHooks::default() | ActiveHooks::FILTER_INTERSECTION_PAIR | ActiveHooks::FILTER_CONTACT_PAIRS); }
                SimObjectType::Kinematic => { collider.set_active_hooks(ActiveHooks::default() | ActiveHooks::FILTER_INTERSECTION_PAIR | ActiveHooks::FILTER_CONTACT_PAIRS); }
            }
        }
    }

    ///
    /// Updates the collider handle for this object
    ///
    pub fn set_collider_handle(&mut self, collider_id: ColliderHandle, collider_set: &mut ColliderSet, islands: &mut IslandManager, rigid_bodies: &mut RigidBodySet) {
        if let Some(old_collider) = self.collider_handle.take() {
            collider_set.remove(old_collider, islands, rigid_bodies, true);
        }

        self.collider_handle = Some(collider_id);

        if let Some(collider) = self.collider_handle.and_then(|handle| collider_set.get_mut(handle)) {
            match self.body_type {
                SimObjectType::Static    => { collider.set_active_hooks(ActiveHooks::default()); }
                SimObjectType::Dynamic   => { collider.set_active_hooks(ActiveHooks::default() | ActiveHooks::FILTER_INTERSECTION_PAIR | ActiveHooks::FILTER_CONTACT_PAIRS); }
                SimObjectType::Kinematic => { collider.set_active_hooks(ActiveHooks::default() | ActiveHooks::FILTER_INTERSECTION_PAIR | ActiveHooks::FILTER_CONTACT_PAIRS); }
            }
        }
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
                SimObjectType::Static       => { /* rigid_body.set_position(Isometry::new(vector![new_position.x() as _, new_position.y() as _], 0.0), true); */ }
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

    ///
    /// Performs an action when the bindings of this object change
    ///
    pub fn when_changed(&mut self, notify: Arc<dyn Notifiable>) {
        // Stop any existing notification
        if let Some(bindings_changed) = self.bindings_changed.take() {
            bindings_changed.lock().unwrap().done();
        }

        // Create a new notification, if there are any dependencies
        let mut dependencies     = BindingDependencies::new();
        let mut num_dependencies = 0;

        if let Some(position) = &mut self.position {
            dependencies.add_dependency(position.clone());
            num_dependencies += 1;
        }

        if let Some(impulse) = &mut self.impulse {
            dependencies.add_dependency(impulse.clone());
            num_dependencies += 1;
        }

        // If there are any dependencies, set up the notifications
        if num_dependencies > 0 {
            self.bindings_changed = Some(Mutex::new(dependencies.when_changed(notify)));
        }
    }
}

impl SimObjectCollection {
    ///
    /// Creates an empty collection
    ///
    pub fn new() -> Self {
        SimObjectCollection { 
            bodies:         HashMap::new(),
            id_for_handle:  HashMap::new(),
        }
    }

    ///
    /// Adds an object to this collection
    ///
    #[inline]
    pub fn insert(&mut self, object_id: SimObjectId, object: SimObject) {
        let handle = object.rigid_body_handle;
        self.bodies.insert(object_id, object);
        self.id_for_handle.insert(handle, object_id);
    }

    ///
    /// Retrieves the object with the specified ID
    ///
    #[inline]
    pub fn get(&self, object_id: &SimObjectId) -> Option<&SimObject> {
        self.bodies.get(object_id)
    }

    ///
    /// Retrievs the object with the specified ID
    ///
    #[inline]
    pub fn get_mut(&mut self, object_id: &SimObjectId) -> Option<&mut SimObject> {
        self.bodies.get_mut(object_id)
    }

    ///
    /// Removes an object from this collection
    ///
    #[inline]
    pub fn remove(&mut self, object_id: &SimObjectId) -> Option<SimObject> {
        // Remove the object
        let removed_object = self.bodies.remove(object_id)?;

        // Remove the handle
        self.id_for_handle.remove(&removed_object.rigid_body_handle);

        Some(removed_object)
    }

    ///
    /// Retrieves an object by handle
    ///
    #[inline]
    pub fn get_by_handle(&self, handle: RigidBodyHandle) -> Option<&SimObject> {
        self.id_for_handle.get(&handle)
            .and_then(|object_id| self.bodies.get(object_id))
    }

    ///
    /// Retrieves an object by handle
    ///
    #[inline]
    pub fn get_mut_by_handle(&mut self, handle: RigidBodyHandle) -> Option<&mut SimObject> {
        self.id_for_handle.get(&handle)
            .and_then(|object_id| self.bodies.get_mut(object_id))
    }
}

impl PhysicsHooks for SimObjectCollection {
    fn filter_contact_pair(&self, context: &PairFilterContext) -> Option<SolverFlags> {
        // Retrieve the objects that are colliding
        let Some(object1) = context.rigid_body1.and_then(|handle| self.get_by_handle(handle)) else { return Some(SolverFlags::COMPUTE_IMPULSES); };
        let Some(object2) = context.rigid_body2.and_then(|handle| self.get_by_handle(handle)) else { return Some(SolverFlags::COMPUTE_IMPULSES); };

        // If the other object appears in either set of exclusions then these objects don't collide
        if let Some(exclusions) = object1.collision_exclusions.as_ref().map(|exclusions| exclusions.get()) {
            if exclusions.contains(&object2.object_id) { return None; }
        }
        if let Some(exclusions) = object2.collision_exclusions.as_ref().map(|exclusions| exclusions.get()) {
            if exclusions.contains(&object1.object_id) { return None; }
        }

        Some(SolverFlags::COMPUTE_IMPULSES)
    }

    fn filter_intersection_pair(&self, context: &PairFilterContext) -> bool {
        // Retrieve the objects that are colliding
        let Some(object1) = context.rigid_body1.and_then(|handle| self.get_by_handle(handle)) else { return true; };
        let Some(object2) = context.rigid_body2.and_then(|handle| self.get_by_handle(handle)) else { return true; };

        // If the other object appears in either set of exclusions then these objects don't collide
        if let Some(exclusions) = object1.collision_exclusions.as_ref().map(|exclusions| exclusions.get()) {
            if exclusions.contains(&object2.object_id) { return false; }
        }
        if let Some(exclusions) = object2.collision_exclusions.as_ref().map(|exclusions| exclusions.get()) {
            if exclusions.contains(&object1.object_id) { return false; }
        }

        true
    }

    fn modify_solver_contacts(&self, _context: &mut ContactModificationContext) {
    }
}
