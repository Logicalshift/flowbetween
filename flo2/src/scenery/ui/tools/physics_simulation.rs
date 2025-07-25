//!
//! Subprogram that provides a general-purpose physics simulation. Bindings are passed in as a way for physics
//! objects to communicate their positions, so this doesn't generate any messages by itself.
//!
//! See `NotifySubprogram` as a way to generate events from bindings (usually you'd use this with something
//! like a rendering algorithm to show where the objects are)
//!

use crate::scenery::ui::ui_path::*;

use flo_binding::*;
use flo_scene::*;
use flo_scene::programs::*;
use flo_curves::*;

use rapier2d::prelude::*;
use uuid::*;
use ::serde::*;
use ::serde::de::{Error as DeError};
use ::serde::ser::{Error as SeError};
use futures::prelude::*;

use std::collections::{HashMap, HashSet};
use std::time::{Duration};

/// Time per tick/physics step
const TICK_DURATION_S: f64 = 1.0/60.0;

/// Maximum number of ticks to process in one iteration (if the scene gets stalled or time otherwise passes in a non-linear fashion)
const MAX_TICKS: usize = 30;

///
/// Identifier used to specify a physics tool within the flowbetween app
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimulationObjectId(Uuid);

impl SimulationObjectId {
    ///
    /// Creates a unique new physics tool ID
    ///
    pub fn new() -> Self {
        SimulationObjectId(Uuid::new_v4())
    }
}

///
/// Message that can be used to control a physics simulation
///
pub enum PhysicsSimulation {
    /// Indicates that time has passed. The duration is the time since the simulation was started
    ///
    /// Physics simulations have clocks that run externally (this allows them to be used for realtime and non-realtime tasks)
    Tick(Duration),

    /// Creates a new rigid body in this simulation
    CreateRigidBody(SimulationObjectId),

    /// Removes a rigit body from this simulation
    RemoveRigidBody(SimulationObjectId),

    /// Sets a property associated with a rigit body
    Set(SimulationObjectId, Vec<PhysicsRigidBodyProperty>),

    /// Specifies a binding that will update when a simulated object moves
    BindPosition(SimulationObjectId, Binding<UiPoint>),

    /// Specifies a binding that will be updated when the angle of an object changes
    BindAngle(SimulationObjectId, Binding<f64>),

    /// Specifies a binding that will update when the velocity of an object changes
    BindVelocity(SimulationObjectId, Binding<UiPoint>),

    /// Specifies a binding that will update when the angular velocity of an object changes
    BindAngularVelocity(SimulationObjectId, Binding<f64>),
}

///
/// The properties that can be assigned to a rigit body
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PhysicsRigidBodyProperty {
    Position(UiPoint),
    Velocity(UiPoint),
    AngularVelocity(f64),
    Shape(SimulationShape),
    Type(SimulationObjectType),
}

///
/// Shapes permitted by a simulation object
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationShape {
    /// The object has no collision
    None,

    /// A circle of the specified radius
    Circle(f64),
}

///
/// Shapes permitted by a simulation object
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationObjectType {
    /// Object that does not move
    Static,

    /// Object that moves using the simulation (when MoveTo is called, we move this object by applying a force to it rather than teleporting it)
    Dynamic,

    /// Object that can have its coordiantes set immediately
    Kinematic,
}

///
/// Physics simualations can generate a few events
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicsSimulationEvent {
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
    let mut physics_events  = context.send::<PhysicsSimulationEvent>(()).unwrap();

    // If we're managing our own timer, then request ticks from the timer program
    if manage_timer {
        timer_program.as_mut().unwrap()
            .send(TimerRequest::CallEvery(our_program_id, 0, Duration::from_micros((TICK_DURATION_S * 1_000_000.0) as _)))
            .await
            .unwrap();
        timer_awake = true;
    }

    // Our own state
    let mut object_id_for_rigid_body_id = HashMap::new();
    let mut rigid_body_id_for_object_id = HashMap::new();
    let mut collider_id_for_object_id   = HashMap::new();
    let mut rigid_body_type             = HashMap::new();
    let mut new_objects                 = HashSet::new();

    // Create the rapier2d state
    let gravity                 = vector![0.0, -9.81];
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
    let physics_hooks           = ();
    let event_handler           = ();

    // Bindings for the objects
    let mut position_bindings           = HashMap::new();
    let mut angle_bindings              = HashMap::new();
    let mut velocity_bindings           = HashMap::new();
    let mut angular_velocity_bindings   = HashMap::new();

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

                // Store the handle and type for this object
                object_id_for_rigid_body_id.insert(handle, object_id);
                rigid_body_id_for_object_id.insert(object_id, handle);
                rigid_body_type.insert(object_id, SimulationObjectType::Kinematic);
                new_objects.insert(object_id);
            },

            RemoveRigidBody(object_id) => {
                if let Some(handle) = rigid_body_id_for_object_id.get(&object_id) {
                    rigid_body_set.remove(*handle, &mut island_manager, &mut collider_set, &mut impulse_joint_set, &mut multibody_joint_set, true);

                    object_id_for_rigid_body_id.remove(handle);
                    rigid_body_id_for_object_id.remove(&object_id);
                    collider_id_for_object_id.remove(&object_id);
                    rigid_body_type.remove(&object_id);

                    position_bindings.remove(&object_id);
                    angle_bindings.remove(&object_id);
                    velocity_bindings.remove(&object_id);
                    angular_velocity_bindings.remove(&object_id);
                    new_objects.remove(&object_id);
                }
            }

            Set(object_id, properties) => {
                // Fetch the object that the property is for
                if let Some(handle) = rigid_body_id_for_object_id.get(&object_id) {
                    // Set this property
                    use PhysicsRigidBodyProperty::*;
                    for property in properties.into_iter() {
                        let rigid_body = rigid_body_set.get_mut(*handle).unwrap();

                        match property {
                            Velocity(velocity)                      => { rigid_body.set_linvel(vector![velocity.x() as _, velocity.y() as _], true); }
                            AngularVelocity(velocity)               => { rigid_body.set_angvel(velocity as _, true); }
                            Type(SimulationObjectType::Static)      => { rigid_body.set_body_type(RigidBodyType::Fixed, true); rigid_body_type.insert(object_id, SimulationObjectType::Static); }
                            Type(SimulationObjectType::Dynamic)     => { rigid_body.set_body_type(RigidBodyType::Dynamic, true); rigid_body_type.insert(object_id, SimulationObjectType::Dynamic); }
                            Type(SimulationObjectType::Kinematic)   => { rigid_body.set_body_type(RigidBodyType::KinematicPositionBased, true); rigid_body_type.insert(object_id, SimulationObjectType::Kinematic); }

                            Shape(SimulationShape::None)            => {
                                if let Some(collider_id) = collider_id_for_object_id.get(&object_id) {
                                    collider_set.remove(*collider_id, &mut island_manager, &mut rigid_body_set, true);
                                }

                                collider_id_for_object_id.remove(&object_id);
                            },

                            Shape(SimulationShape::Circle(radius)) => {
                                // Remove any existing colliders
                                if let Some(collider_id) = collider_id_for_object_id.get(&object_id) {
                                    collider_set.remove(*collider_id, &mut island_manager, &mut rigid_body_set, true);
                                }

                                if let Some(rigid_body_handle) = rigid_body_id_for_object_id.get(&object_id) {
                                    let rigid_body  = rigid_body_set.get(*rigid_body_handle).unwrap();
                                    let position    = rigid_body.position().translation;

                                    // Create a ball collider
                                    let collider = ColliderBuilder::ball(radius as _)
                                        .translation(position.vector)
                                        .build();

                                    let collider_id = collider_set.insert_with_parent(collider, *rigid_body_handle, &mut rigid_body_set);
                                    collider_id_for_object_id.insert(object_id, collider_id);
                                }
                            }

                            Position(pos) => {
                                if new_objects.contains(&object_id) {
                                    // Set the position immediately if the body is new
                                    rigid_body.set_position(Isometry::new(vector![pos.x() as _, pos.y() as _], 0.0), true);
                                } else {
                                    // Action for setting the position depends on the type of the object
                                    // TODO: dynamic objects need to be 'pushed' into their intended position
                                    match rigid_body_type.get(&object_id) {
                                        None                                    |
                                        Some(SimulationObjectType::Static)      => { rigid_body.set_position(Isometry::new(vector![pos.x() as _, pos.y() as _], 0.0), true); }
                                        Some(SimulationObjectType::Dynamic)     => { rigid_body.set_position(Isometry::new(vector![pos.x() as _, pos.y() as _], 0.0), true); }
                                        Some(SimulationObjectType::Kinematic)   => { rigid_body.set_next_kinematic_position(Isometry::new(vector![pos.x() as _, pos.y() as _], 0.0)); rigid_body.wake_up(false); }
                                    }
                                }
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

            BindPosition(object_id, binding)        => { if rigid_body_id_for_object_id.contains_key(&object_id) { position_bindings.insert(object_id, binding); } },
            BindAngle(object_id, binding)           => { if rigid_body_id_for_object_id.contains_key(&object_id) { angle_bindings.insert(object_id, binding); } },
            BindVelocity(object_id, binding)        => { if rigid_body_id_for_object_id.contains_key(&object_id) { velocity_bindings.insert(object_id, binding); } },
            BindAngularVelocity(object_id, binding) => { if rigid_body_id_for_object_id.contains_key(&object_id) { angular_velocity_bindings.insert(object_id, binding); } },

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

                // There are no more new objects
                new_objects.clear();

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
                        &physics_hooks,
                        &event_handler,
                    );
                }

                // Update the bindings
                let mut some_awake = false;

                for (handle, body) in rigid_body_set.iter() {
                    let object_id = if let Some(object_id) = object_id_for_rigid_body_id.get(&handle) { object_id } else { continue; };

                    // If any rigid bodies are awake, then the simulation should also be kept awake
                    if !body.is_sleeping() {
                        some_awake = true;
                    }

                    // Set any bindings relating to this object
                    if let Some(position) = position_bindings.get(object_id) { 
                        let body_pos = body.position().translation;
                        position.set(UiPoint(body_pos.vector[0] as _, body_pos.vector[1] as _))
                    }

                    if let Some(angle) = angle_bindings.get(object_id) {
                        let body_angle = body.rotation().angle();
                        angle.set(body_angle as _);
                    }

                    if let Some(velocity) = velocity_bindings.get(object_id) {
                        let body_velocity = body.vels();
                        velocity.set(UiPoint(body_velocity.linvel[0] as _, body_velocity.linvel[1] as _));
                    }

                    if let Some(angular_velocity) = angular_velocity_bindings.get(object_id) {
                        let body_angular_velocity = body.vels().angvel;
                        angular_velocity.set(body_angular_velocity as _);
                    }
                }

                // Send the 'step' event
                physics_events.send(PhysicsSimulationEvent::Step).await.ok();

                // Go to sleep if no rigid bodies are awake
                if !some_awake {
                    // Indicate that everything is asleep (can stop generating physics events)
                    physics_events.send(PhysicsSimulationEvent::Sleep).await.ok();

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
}

impl SceneMessage for PhysicsSimulationEvent {

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
