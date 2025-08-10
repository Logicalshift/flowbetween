//!
//! Subprogram that provides a general-purpose physics simulation. Bindings are passed in as a way for physics
//! objects to communicate their positions, so this doesn't generate any messages by itself.
//!
//! See `NotifySubprogram` as a way to generate events from bindings (usually you'd use this with something
//! like a rendering algorithm to show where the objects are)
//!

use super::physics_simulation_joints::*;
use super::physics_simulation_object::*;
use crate::scenery::ui::ui_path::*;
use crate::scenery::ui::binding_tracker::*;

use flo_binding::*;
use flo_scene::*;
use flo_scene::programs::*;
use flo_curves::*;

use rapier2d::prelude::*;
use ::serde::*;
use ::serde::de::{Error as DeError};
use ::serde::ser::{Error as SeError};
use futures::prelude::*;

use std::collections::{HashMap, HashSet};
use std::time::{Instant, Duration};

/// Time per tick/physics step
const TICK_DURATION_S: f64 = 1.0/60.0;

/// Maximum number of ticks to process in one iteration (if the scene gets stalled or time otherwise passes in a non-linear fashion)
const MAX_TICKS: usize = 30;

///
/// Message that can be used to control a physics simulation
///
pub enum PhysicsSimulation {
    /// Indicates that time has passed. The duration is the time since the simulation was started
    ///
    /// Physics simulations have clocks that run externally (this allows them to be used for realtime and non-realtime tasks)
    Tick(Duration),

    /// Indicates that the properties of an object have been updated (this is triggered automatically when any of the binding change)
    UpdateObject(SimObjectId),

    /// Creates a new rigid body in this simulation
    CreateRigidBody(SimObjectId),

    /// Removes a rigid body from this simulation
    RemoveRigidBody(SimObjectId),

    /// Sets a property associated with a rigid body
    Set(SimObjectId, Vec<SimBodyProperty>),

    /// Sets one or more global properties for the simulation
    SetGlobal(Vec<SimGlobalProperty>),

    /// Specifies a binding that will update when a simulated object moves
    BindPosition(SimObjectId, Binding<UiPoint>),

    /// Specifies a binding that will be updated when the angle of an object changes
    BindAngle(SimObjectId, Binding<f64>),

    /// Specifies a binding that will update when the velocity of an object changes
    BindVelocity(SimObjectId, Binding<UiPoint>),

    /// Specifies a binding that will update when the angular velocity of an object changes
    BindAngularVelocity(SimObjectId, Binding<f64>),

    /// Adds a new joint connecting two objects
    AddJoint(SimJointId, SimObjectId, SimObjectId, SimJoint),

    /// Updates the properties of a joint
    SetJoint(SimJointId, Vec<SimJointProperty>),

    /// Removes a joint (joints are also removed if either of the objects they connect are removed)
    RemoveJoint(SimJointId),
}

///
/// The properties that can be assigned to a rigid body
///
#[derive(Clone)]
pub enum SimBodyProperty {
    /// The position of the object. Kinematic objects can have their position set instantaneously, dynamic 
    /// objects use a spring to bind them to this position, static object teleport.
    ///
    /// The position here is set as a binding, so the desired position of the object can be updated by
    /// setting the bound value instead of sending property update events.
    Position(Option<BindRef<UiPoint>>),

    /// The velocity of this object
    Velocity(UiPoint),

    /// The angular velocity of this object
    AngularVelocity(f64),

    /// How the linear velocity of this object is damped
    LinearDamping(f64),

    /// How the angular damping of this object is damped
    AngularDamping(f64),

    /// If true, the object is not permitted to rotate
    LockRotation(bool),

    /// The shape of this object
    Shape(SimShape),

    /// The type of this object (how it interacts with the world)
    Type(SimObjectType),

    /// The impulse that is applied to this object (calculated every frame, wakes the simulation if updated)
    BindImpulse(Option<BindRef<UiPoint>>),

    /// The objects that this object will not collide with
    IgnoreCollisions(BindRef<Vec<SimObjectId>>),
}

///
/// The properties that apply to the simulation as a whole
///
pub enum SimGlobalProperty {
    /// Sets the simulation gravity
    Gravity(UiPoint),
}

///
/// Shapes permitted by a simulation object
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimShape {
    /// The object has no collision
    None,

    /// A circle of the specified radius
    Circle(f64),
}

///
/// Physics simualations can generate a few events
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicsSimEvent {
    /// The simulation has reached a steady state and doesn't need to be woken again
    Sleep,

    /// One or more steps have been simulated
    Step,
}

///
/// Runs a physics simulation using rapier2d
///
/// The bindings can be used to receive updates (eg, `PhysicsSimulation::BindPosition()`) can be used to track where
/// the objects are.
///
/// If `manage_timer` is set to true, this program will request timer events from the standard flo_scene timer program.
/// If it's false, then the simulation will only advance when the `Tick()` event is generated manually (which can be
/// useful for tests)
///
pub async fn physics_simulation_program(input: InputStream<PhysicsSimulation>, context: SceneContext, manage_timer: bool) {
    let our_program_id  = context.current_program_id().unwrap();
    let mut input       = input;

    // Connect messages
    let mut timer_awake     = false;
    let mut timer_program   = if manage_timer { context.send::<TimerRequest>(()).ok() } else { None };
    let mut physics_events  = context.send::<PhysicsSimEvent>(()).unwrap();

    // If we're managing our own timer, then request ticks from the timer program
    if manage_timer {
        timer_program.as_mut().unwrap()
            .send(TimerRequest::CallEvery(our_program_id, 0, Duration::from_micros((TICK_DURATION_S * 1_000_000.0) as _)))
            .await
            .unwrap();
        timer_awake = true;
    }

    // Our own state
    let mut rigid_bodies        = SimObjectCollection::new();
    let mut new_objects         = HashSet::new();
    let mut joints              = HashMap::new();
    let mut recently_changed    = HashSet::new();

    // Create the rapier2d state
    let mut gravity             = vector![0.0, 9.81];
    let mut rigid_body_set      = RigidBodySet::new();
    let mut collider_set        = ColliderSet::new();
    let integration_parameters  = IntegrationParameters::default();
    let mut physics_pipeline    = PhysicsPipeline::new();
    let mut island_manager      = IslandManager::new();
    let mut broad_phase         = DefaultBroadPhase::new();
    let mut narrow_phase        = NarrowPhase::new();
    let mut impulse_joint_set   = ImpulseJointSet::new();
    let mut multibody_joint_set = MultibodyJointSet::new();
    let mut ccd_solver          = CCDSolver::new();
    let mut query_pipeline      = QueryPipeline::new();
    let event_handler           = ();

    // We track time from 0. Time doesn't pass while we're asleep
    let mut last_step_time  = Duration::default();
    let mut is_asleep       = true;

    while let Some(event) = input.next().await {
        use PhysicsSimulation::*;

        match event {
            CreateRigidBody(object_id) => {
                // Create a kinematic body with no properties (everything is kinematic by default so the position can be set easily)
                let rigid_body  = RigidBodyBuilder::kinematic_position_based().build();
                let handle      = rigid_body_set.insert(rigid_body);
                let object      = SimObject::kinematic_position_based(object_id, handle);

                // Store the handle and type for this object
                rigid_bodies.insert(object_id, object);
                new_objects.insert(object_id);
            },

            Set(object_id, properties) => {
                // Fetch the object that the property is for
                if let Some(object) = rigid_bodies.get_mut(&object_id) {
                    // Set this property
                    use SimBodyProperty::*;
                    for property in properties.into_iter() {
                        let rigid_body = rigid_body_set.get_mut(object.rigid_body_handle).unwrap();

                        match property {
                            Velocity(velocity)              => { rigid_body.set_linvel(vector![velocity.x() as _, velocity.y() as _], true); }
                            AngularVelocity(velocity)       => { rigid_body.set_angvel(velocity as _, true); }
                            LinearDamping(damping)          => { rigid_body.set_linear_damping(damping as _); }
                            AngularDamping(damping)         => { rigid_body.set_angular_damping(damping as _); }
                            LockRotation(is_locked)         => { rigid_body.lock_rotations(is_locked, true); }
                            Type(object_type)               => { object.set_body_type(object_type, &mut rigid_body_set, &mut collider_set); }

                            Shape(SimShape::None) => {
                                if let Some(collider_id) = object.collider_handle {
                                    collider_set.remove(collider_id, &mut island_manager, &mut rigid_body_set, true);
                                }

                                object.collider_handle = None;
                            },

                            Shape(SimShape::Circle(radius)) => {
                                // Remove any existing colliders
                                if let Some(collider_id) = object.collider_handle {
                                    collider_set.remove(collider_id, &mut island_manager, &mut rigid_body_set, true);
                                    object.collider_handle = None;
                                }

                                // Create a ball collider
                                let collider = ColliderBuilder::ball(radius as _)
                                    .build();

                                let collider_id = collider_set.insert_with_parent(collider, object.rigid_body_handle, &mut rigid_body_set);
                                object.collider_handle = Some(collider_id);
                            }

                            Position(pos) => {
                                object.position = pos;
                                object.when_changed(NotifySubprogram::send(PhysicsSimulation::UpdateObject(object_id), &context, our_program_id));
                                recently_changed.insert(object_id);
                            }

                            BindImpulse(impulse_binding) => {
                                object.impulse = impulse_binding;
                                object.when_changed(NotifySubprogram::send(PhysicsSimulation::UpdateObject(object_id), &context, our_program_id));
                                recently_changed.insert(object_id);
                            }

                            IgnoreCollisions(collision_binding) => {
                                object.collision_exclusions = Some(collision_binding);
                                object.when_changed(NotifySubprogram::send(PhysicsSimulation::UpdateObject(object_id), &context, our_program_id));
                                recently_changed.insert(object_id);
                            }
                        }
                    }
                }

                // Wake up the timer again whenever any property is set
                if manage_timer && !timer_awake {
                    timer_program.as_mut().unwrap()
                        .send(TimerRequest::CallEvery(our_program_id, 0, Duration::from_micros((TICK_DURATION_S * 1_000_000.0) as _)))
                        .await
                        .unwrap();
                    timer_awake = true;
                }
            }

            SetGlobal(properties) => {
                for property in properties {
                    match property {
                        SimGlobalProperty::Gravity(UiPoint(x, y)) => {
                            gravity = vector![x as _, y as _];
                        }
                    }
                }
            }

            UpdateObject(object_id) => {
                // Add to the recently changed list
                recently_changed.insert(object_id);

                // Make sure the simulation is awake
                if manage_timer && !timer_awake {
                    timer_program.as_mut().unwrap()
                        .send(TimerRequest::CallEvery(our_program_id, 0, Duration::from_micros((TICK_DURATION_S * 1_000_000.0) as _)))
                        .await
                        .unwrap();
                    timer_awake = true;
                }
            }

            AddJoint(joint_id, object_1_id, object_2_id, joint) => {
                if let (Some(object_1), Some(object_2)) = (rigid_bodies.get(&object_1_id), rigid_bodies.get(&object_2_id)) {
                    // Create the joint
                    let joint = joint.create();

                    // Add to the set of joints
                    let joint_handle = impulse_joint_set.insert(object_1.rigid_body_handle, object_2.rigid_body_handle, joint, true);
                    joints.insert(joint_id, joint_handle);

                    // Also associate with the object, so we can remove it later on if the object is removed
                    rigid_bodies.get_mut(&object_1_id).unwrap().joints.push(joint_id);
                    rigid_bodies.get_mut(&object_2_id).unwrap().joints.push(joint_id);
                }

                // Wake up the timer again whenever a joint is added
                if manage_timer && !timer_awake {
                    timer_program.as_mut().unwrap()
                        .send(TimerRequest::CallEvery(our_program_id, 0, Duration::from_micros((TICK_DURATION_S * 1_000_000.0) as _)))
                        .await
                        .unwrap();
                    timer_awake = true;
                }
            }

            SetJoint(joint_id, joint_properties) => { 
                if let Some(joint_handle) = joints.get(&joint_id).copied() {
                    let joint = impulse_joint_set.get_mut(joint_handle, true).unwrap();

                    for property in joint_properties {
                        match property {
                            SimJointProperty::ContactsEnabled(val)                      => { joint.data.contacts_enabled = val; }
                            SimJointProperty::LocalAnchor(SimJointSide::First, offset)  => { joint.data.local_frame1 = Isometry::translation(offset.x() as _, offset.y() as _)}
                            SimJointProperty::LocalAnchor(SimJointSide::Second, offset) => { joint.data.local_frame2 = Isometry::translation(offset.x() as _, offset.y() as _)}
                        }
                    }
                }

                // Wake up the timer again whenever a joint's properties are changed
                if manage_timer && !timer_awake {
                    timer_program.as_mut().unwrap()
                        .send(TimerRequest::CallEvery(our_program_id, 0, Duration::from_micros((TICK_DURATION_S * 1_000_000.0) as _)))
                        .await
                        .unwrap();
                    timer_awake = true;
                }
            }

            RemoveJoint(joint_id) => {
                if let Some(joint_handle) = joints.get(&joint_id).copied() {
                    let joint = impulse_joint_set.get(joint_handle).unwrap();

                    // Remove the structures
                    let object_1 = rigid_bodies.get_mut_by_handle(joint.body1);
                    if let Some(object_1) = object_1 { object_1.joints.retain(|candidate| candidate != &joint_id); }

                    let object_2 = rigid_bodies.get_mut_by_handle(joint.body2);
                    if let Some(object_2) = object_2 { object_2.joints.retain(|candidate| candidate != &joint_id); }

                    // Finished with the joint
                    joints.remove(&joint_id);
                    impulse_joint_set.remove(joint_handle, true);
                }

                // Wake up the timer again whenever a joint is deleted
                if manage_timer && !timer_awake {
                    timer_program.as_mut().unwrap()
                        .send(TimerRequest::CallEvery(our_program_id, 0, Duration::from_micros((TICK_DURATION_S * 1_000_000.0) as _)))
                        .await
                        .unwrap();
                    timer_awake = true;
                }
            }

            RemoveRigidBody(object_id) => {
                if let Some(object) = rigid_bodies.get(&object_id) {
                    // Remove from the simulation
                    rigid_body_set.remove(object.rigid_body_handle, &mut island_manager, &mut collider_set, &mut impulse_joint_set, &mut multibody_joint_set, true);

                    // Remove the joints attached to this object
                    let mut joints_to_remove = vec![];
                    for joint_id in object.joints.iter().copied() {
                        if let Some(joint_handle) = joints.get(&joint_id).copied() {
                            let joint = impulse_joint_set.get(joint_handle).unwrap();

                            // Remove the structures
                            if joint.body1 != object.rigid_body_handle {
                                joints_to_remove.push((joint.body1, joint_id));
                            } else {
                                joints_to_remove.push((joint.body2, joint_id));
                            }

                            // Finished with the joint
                            joints.remove(&joint_id);
                            impulse_joint_set.remove(joint_handle, true);
                        }
                    }

                    // Remove the spring binding this object to its position
                    if let Some((old_anchor, old_spring)) = object.anchor_joint {
                        impulse_joint_set.remove(old_spring, true);
                        rigid_body_set.remove(old_anchor, &mut island_manager, &mut collider_set, &mut impulse_joint_set, &mut multibody_joint_set, true);
                    }

                    // Remove the IDs and references to this object
                    rigid_bodies.remove(&object_id);

                    // Tidy up the other side of the joints
                    for (rigid_body_handle, joint_id) in joints_to_remove {
                        let Some(object) = rigid_bodies.get_mut_by_handle(rigid_body_handle) else { continue; };
                        object.joints.retain(|candidate| candidate != &joint_id);
                    }
                }
            }

            BindPosition(object_id, binding)        => { if let Some(object) = rigid_bodies.get_mut(&object_id) { object.state_bindings.position         = Some(binding); } },
            BindAngle(object_id, binding)           => { if let Some(object) = rigid_bodies.get_mut(&object_id) { object.state_bindings.angle            = Some(binding); } },
            BindVelocity(object_id, binding)        => { if let Some(object) = rigid_bodies.get_mut(&object_id) { object.state_bindings.velocity         = Some(binding); } },
            BindAngularVelocity(object_id, binding) => { if let Some(object) = rigid_bodies.get_mut(&object_id) { object.state_bindings.angular_velocity = Some(binding); } },

            Tick(time) => {
                let tick = Duration::from_micros((TICK_DURATION_S * 1_000_000.0) as _);

                if is_asleep {
                    // Wake up if we're asleep (don't run a bunch of steps if we've been asleep for a while)
                    is_asleep       = false;
                    last_step_time  = if time >= tick { time - tick } else { time };
                }

                if time < last_step_time {
                    // Something is wrong with the timer, and it's moved backwards (might happen if we went to sleep and woke up again)
                    last_step_time = if time >= tick { time - tick } else { time };
                }

                // Update the status of any object in the 'recently changed' list
                for changed_object_id in recently_changed.drain() {
                    let Some(object) = rigid_bodies.get_mut(&changed_object_id) else { continue; };

                    if let Some(position) = object.position.as_ref().map(|pos| pos.get()) {
                        object.update_position(position, &mut rigid_body_set, &mut impulse_joint_set);
                    }

                    if let Some(impulse) = object.impulse.as_ref().map(|impulse| impulse.get()) {
                        let body = rigid_body_set.get_mut(object.rigid_body_handle);
                        if let Some(body) = body {
                            body.add_force(vector![impulse.x() as _, impulse.y() as _], true);
                        }
                    }

                    object.when_changed(NotifySubprogram::send(UpdateObject(changed_object_id), &context, our_program_id));
                }

                // There are no more new objects
                for new_object in new_objects.drain() {
                    let Some(object) = rigid_bodies.get_mut(&new_object) else { continue; };
                    object.is_new = false;
                }

                // Run up to MAX_TICKS steps
                let mut num_steps   = 0;
                loop {
                    if num_steps >= MAX_TICKS {
                        // Just make up the remaining time and stop
                        last_step_time = time;
                        break;
                    }

                    if last_step_time + tick > time {
                        // Stop when we can't run a full tick in the time we've got left
                        break;
                    }

                    // Add in the tick time
                    last_step_time  += tick;
                    num_steps       += 1;

                    // Run the step
                    physics_pipeline.step(
                        &gravity,
                        &integration_parameters,
                        &mut island_manager,
                        &mut broad_phase,
                        &mut narrow_phase,
                        &mut rigid_body_set,
                        &mut collider_set,
                        &mut impulse_joint_set,
                        &mut multibody_joint_set,
                        &mut ccd_solver,
                        Some(&mut query_pipeline),
                        &rigid_bodies,
                        &event_handler,
                    );
                }

                // Update the bindings
                let mut some_awake = false;

                for (handle, body) in rigid_body_set.iter() {
                    let Some(object) = rigid_bodies.get_mut_by_handle(handle) else { continue; };

                    // If any rigid bodies are awake, then the simulation should also be kept awake
                    if !body.is_sleeping() {
                        some_awake = true;
                    }

                    // Set any bindings relating to this object
                    if let Some(position) = &object.state_bindings.position { 
                        let body_pos = body.position().translation;
                        position.set(UiPoint(body_pos.vector[0] as _, body_pos.vector[1] as _))
                    }

                    if let Some(angle) = &object.state_bindings.angle {
                        let body_angle = body.rotation().angle();
                        angle.set(body_angle as _);
                    }

                    if let Some(velocity) = &object.state_bindings.velocity {
                        let body_velocity = body.vels();
                        velocity.set(UiPoint(body_velocity.linvel[0] as _, body_velocity.linvel[1] as _));
                    }

                    if let Some(angular_velocity) = &object.state_bindings.angular_velocity {
                        let body_angular_velocity = body.vels().angvel;
                        angular_velocity.set(body_angular_velocity as _);
                    }
                }

                // Send the 'step' event
                physics_events.send(PhysicsSimEvent::Step).await.ok();

                // Go to sleep if no rigid bodies are awake
                if !some_awake {
                    // Indicate that everything is asleep (can stop generating physics events)
                    physics_events.send(PhysicsSimEvent::Sleep).await.ok();

                    // If we're managing our own timer, then put that to sleep (note that there can be extra ticks waiting to be delivered if we're behind)
                    if manage_timer && timer_awake {
                        timer_program.as_mut().unwrap()
                            .send(TimerRequest::Cancel(our_program_id, 0))
                            .await
                            .unwrap();
                        timer_awake = false;
                    }
                }
            },
        }
    }
}

impl SceneMessage for PhysicsSimulation {
    #[inline]
    fn serializable() -> bool { false }

    fn initialise(init_context: &impl SceneInitialisationContext) {
        init_context.connect_programs((), StreamTarget::None, StreamId::with_message_type::<PhysicsSimEvent>()).unwrap();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|timeout_events| {
            let start_time = Instant::now();

            timeout_events.map(move |_timeout: TimeOut| {
                let elapsed_time = Instant::now().duration_since(start_time);
                PhysicsSimulation::Tick(elapsed_time)
            })
        })), (), StreamId::with_message_type::<TimeOut>()).unwrap();
    }
}

impl SceneMessage for PhysicsSimEvent {

}

impl Serialize for PhysicsSimulation {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer 
    {
        Err(S::Error::custom("PhysicsLayer cannot be serialized"))
    }
}

impl<'a> Deserialize<'a> for PhysicsSimulation {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a> 
    {
        Err(D::Error::custom("PhysicsLayer cannot be serialized"))
    }
}
