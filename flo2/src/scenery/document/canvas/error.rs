use super::brush::*;
use super::layer::*;
use super::shape::*;

use ::serde::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CanvasError {
    /// Something unexpected went wrong while storing to the canvas
    UnexpectedStorageError(String),

    /// Operation failed because a layer does not exist
    NoSuchLayer(CanvasLayerId),

    /// Operation failed because a shape does not exist
    NoSuchShape(CanvasShapeId),

    /// Operation failed because a brush does not exisat
    NoSuchBrush(CanvasBrushId),
}
