use super::layer::*;

use crate::scenery::ui::*;

use ::serde::*;
use uuid::*;

///
/// Identifier used for a shape in the canvas document
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasShapeId(Uuid);

///
/// Defines a shape on the canvas
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CanvasShape {
    /// Arbitrary bezier path
    Path(UiPath),

    /// Group of other shapes (with no shape itself)
    Group,

    /// Rectangle shape
    Rectangle { min: UiPoint, max: UiPoint },

    /// Ellipse filling a rectangle
    Ellipse { min: UiPoint, max: UiPoint },

    /// Polygon filling a rectangle, with the specified number of points
    Polygon { min: UiPoint, max: UiPoint, points: usize },
}

///
/// Specifies the parent for a canvas shape
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CanvasShapeParent {
    /// Shape is not a parent shape
    None,

    /// Shape is part of a layer
    Layer(CanvasLayerId),

    /// Shape is grouped with another shape
    Shape(CanvasShapeId),
}

impl CanvasShapeId {
    ///
    /// Creates a unique new canvas path ID
    ///
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
