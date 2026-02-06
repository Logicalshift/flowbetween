use super::super::point::*;

use ::serde::*;

///
/// Serialized form of an ellipse in the canvas
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CanvasEllipseV1 {
    pub min: CanvasPoint,
    pub max: CanvasPoint,
}

pub type CanvasEllipse = CanvasEllipseV1;

/// Shape type to indicate a shape encoded in V1 canvas ellipse format
pub const CANVAS_ELLIPSE_V1_TYPE: i64 = 2;
