use super::super::point::*;

use ::serde::*;

///
/// Serialized form of a polygon in the canvas
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CanvasPolygonV1 {
    pub min:        CanvasPoint,
    pub max:        CanvasPoint,
    pub direction:  CanvasPoint,
    pub sides:      usize,
}

pub type CanvasPolygon = CanvasPolygonV1;

/// Shape type to indicate a shape encoded in V1 canvas polygon format
pub const CANVAS_POLYGON_V1_TYPE: i64 = 3;
