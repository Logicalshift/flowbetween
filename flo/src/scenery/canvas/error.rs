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

use super::brush::*;
use super::layer::*;
use super::shape::*;

use flo_scene::*;

use ::serde::*;
use std::error::{Error};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CanvasError {
    /// Something unexpected went wrong while storing to the canvas
    UnexpectedStorageError(String),

    /// An error occurred while serializing or deserializing a string
    SerializationError(String),

    /// An error occurred while connecting to something in a scene
    ConnectionError(ConnectionError),

    /// An error occurred while sending a message in a scene
    SceneSendError(SceneSendError<()>),

    /// Operation failed because a layer does not exist
    NoSuchLayer(CanvasLayerId),

    /// Operation failed because a shape does not exist
    NoSuchShape(CanvasShapeId),

    /// Operation failed because a brush does not exist
    NoSuchBrush(CanvasBrushId),

    /// A shape does not have a parent
    ShapeHasNoParent(CanvasShapeId),
}

impl fmt::Display for CanvasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanvasError::UnexpectedStorageError(msg)    => write!(f, "Unexpected storage error: {}", msg),
            CanvasError::SerializationError(msg)        => write!(f, "Serialization error: {}", msg),
            CanvasError::ConnectionError(err)           => write!(f, "Scene connection error: {:?}", err),
            CanvasError::SceneSendError(err)            => write!(f, "Failed to send message: {:?}", err),
            CanvasError::NoSuchLayer(id)                => write!(f, "No such layer with id: {}", id),
            CanvasError::NoSuchShape(id)                => write!(f, "No such shape with id: {}", id),
            CanvasError::NoSuchBrush(id)                => write!(f, "No such brush with id: {}", id),
            CanvasError::ShapeHasNoParent(id)           => write!(f, "Shape has no parent: {}", id),
        }
    }
}

impl Error for CanvasError { }

impl From<ConnectionError> for CanvasError {
    fn from(value: ConnectionError) -> Self {
        CanvasError::ConnectionError(value)
    }
}

impl<TMsg> From<SceneSendError<TMsg>> for CanvasError {
    fn from(value: SceneSendError<TMsg>) -> Self {
        CanvasError::SceneSendError(value.map(|_| ()))
    }
}
