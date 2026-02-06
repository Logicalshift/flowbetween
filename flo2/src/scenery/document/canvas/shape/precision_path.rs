use super::precision_point::*;

use ::serde::*;

///
/// Represents a subpath of a shape on the canvas
///
/// 'Precision' version, using 64-bit points that's intended for path arithmetic operations
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CanvasPrecisionSubpath {
    /// Initial point of the path
    pub start_point: CanvasPrecisionPoint,

    /// The actions that make up this path
    pub actions: Vec<CanvasPrecisionPoint>,
}

///
/// Actions that can be taken as part of a precision subpath
///
pub enum CanvasPrecisionPathAction {
    /// Line to a specific point
    Line(CanvasPrecisionPoint),

    /// Quadratic bezier curve to the specified point
    QuadraticCurve { end: CanvasPrecisionPoint, cp: CanvasPrecisionPoint },

    /// Cubic bezier curve to a specific point
    CubicCurve { end: CanvasPrecisionPoint, cp1: CanvasPrecisionPoint, cp2: CanvasPrecisionPoint },

    /// Closes the path (generating a line to the start point)
    Close,
}
