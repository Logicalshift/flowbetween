// FlowBetween, a tool for creating vector animations
// Copyright (C) 2026 Andrew Hunter
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::scenery::ui::ui_path::*;

use rapier2d::prelude::*;
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

impl SimJoint {
    ///
    /// Creates this joint
    ///
    pub fn create(&self) -> GenericJoint {
        match self {
            SimJoint::FixedJoint                                        => FixedJoint::new().into(),
            SimJoint::SpringJoint { rest_length, stiffness, damping }   => SpringJointBuilder::new(*rest_length as _, *stiffness as _, *damping as _).spring_model(MotorModel::AccelerationBased).build().into(),
            SimJoint::RopeJoint { max_dist }                            => RopeJointBuilder::new(*max_dist as _).motor_model(MotorModel::AccelerationBased).build().into()
        }
    }
}