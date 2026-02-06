use super::super::point::*;

use ::serde::*;

///
/// Serialized form of a rectangle in the canvas
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CanvasRectangleV1 {
    pub min: CanvasPoint,
    pub max: CanvasPoint,
}

pub type CanvasRectangle = CanvasRectangleV1;

/// Shape type to indicate a shape encoded in V1 canvas rectangle format
pub const CANVAS_RECTANGLE_V1_TYPE: i64 = 1;
