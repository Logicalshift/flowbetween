//!
//! Subprogram that provides a general-purpose physics simulation. Bindings are passed in as a way for physics
//! objects to communicate their positions, so this doesn't generate any messages by itself.
//!
//! See `NotifySubprogram` as a way to generate events from bindings (usually you'd use this with something
//! like a rendering algorithm to show where the objects are)
//!

use crate::scenery::ui::ui_path::*;

use flo_binding::*;

use uuid::*;
use ::serde::*;
use rapier2d::*;

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