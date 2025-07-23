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

use uuid::*;
use ::serde::*;
use ::serde::de::{Error as DeError};
use ::serde::ser::{Error as SeError};
use rapier2d::*;
use futures::prelude::*;

use std::time::{Duration};

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

    /// Sets the position of a simulation object ID
    MoveTo(SimulationObjectId, UiPoint),

    /// Sets the velocity of a simulation object ID
    SetVelocity(SimulationObjectId, UiPoint),

    /// Sets the angular velocity of a simulation object ID
    SetAngularVelocity(SimulationObjectId, f64),

    /// Sets the shape of a simulation object
    SetShape(SimulationObjectId, SimulationShape),

    /// Specifies a binding that will update when a simulated object moves
    BindPosition(SimulationObjectId, Binding<UiPoint>),

    /// Specifies a binding that will be updated when the angle of an object changes
    BindAngle(SimulationObjectId, Binding<f64>),

    /// Specifies a binding that will update when the velocity of an object changes
    BindVelocity(SimulationObjectId, Binding<UiPoint>),

    /// Specifies a binding that will update when the angular velocity of an object changes
    BindAngularVelocity(SimulationObjectId, Binding<UiPoint>),
}

///
/// Shapes permitted by a simulation object
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationShape {
    /// A circle of the specified radius
    Circle(f64),
}

///
/// Physics simualations can generate a few events
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicsSimulationEvent {
    /// The simulation has reached a steady state and doesn't need to be woken again
    Sleep,
}

pub async fn physics_simulation_program(input: InputStream<PhysicsSimulation>, context: SceneContext) {
    let mut input = input;

    while let Some(event) = input.next().await {
        use PhysicsSimulation::*;

        match event {
            Tick(_time)                                 => { },
            CreateRigidBody(_object_id)                 => { },
            MoveTo(_object_id, _pos)                    => { },
            SetVelocity(_object_id, _velocity)          => { },
            SetAngularVelocity(_object_id, _velocity)   => { },
            SetShape(_object_id, _shape)                => { },
            BindPosition(_object_id, _binding)          => { },
            BindAngle(_object_id, _binding)             => { },
            BindVelocity(_object_id, _binding)          => { },
            BindAngularVelocity(_object_id, _binding)   => { },
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
