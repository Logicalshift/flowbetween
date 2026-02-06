use ::serde::*;

///
/// Represents a point on the canvas
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CanvasPoint {
    pub x: f32,
    pub y: f32,
}
