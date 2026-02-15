use super::brush::*;
use super::layer::*;
use super::shape::*;

use ::serde::*;
use std::error::{Error};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CanvasError {
    /// Something unexpected went wrong while storing to the canvas
    UnexpectedStorageError(String),

    /// Operation failed because a layer does not exist
    NoSuchLayer(CanvasLayerId),

    /// Operation failed because a shape does not exist
    NoSuchShape(CanvasShapeId),

    /// Operation failed because a brush does not exist
    NoSuchBrush(CanvasBrushId),
}

impl fmt::Display for CanvasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanvasError::UnexpectedStorageError(msg)    => write!(f, "Unexpected storage error: {}", msg),
            CanvasError::NoSuchLayer(id)                => write!(f, "No such layer with id: {}", id),
            CanvasError::NoSuchShape(id)                => write!(f, "No such shape with id: {}", id),
            CanvasError::NoSuchBrush(id)                => write!(f, "No such brush with id: {}", id),
        }
    }
}

impl Error for CanvasError {}
