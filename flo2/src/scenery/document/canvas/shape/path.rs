use super::super::point::*;

use ::serde::*;

///
/// Serialized form of a bezier path in the canvas
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CanvasPathV1 {
    pub start_point: CanvasPoint,
    pub actions:     Vec<CanvasPathV1Action>,
}

///
/// Actions for each point on a v1 path (except the first point, which is always a 'move' action)
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CanvasPathV1Action {
    /// Draws a line to the start point of the current subpath
    Close,

    /// Starts a subpath
    Move(CanvasPoint),

    /// Draws a line to the specified point
    Line(CanvasPoint),

    /// Creates a quadratic bezier curve to the specified point
    QuadraticCurve { end: CanvasPoint, cp: CanvasPoint },

    /// Creates a cubic bezier curve to the specified point
    CubicCurve { end: CanvasPoint, cp1: CanvasPoint, cp2: CanvasPoint },
}

pub type CanvasPath = CanvasPathV1;

/// Shape type to indicate a shape encoded in V1 canvas path format
pub const CANVAS_PATH_V1_TYPE: i64 = 0;
