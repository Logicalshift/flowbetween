use super::super::layer::*;
use super::ellipse::*;
use super::path::*;
use super::polygon::*;
use super::rectangle::*;

use ::serde::*;
use uuid::*;

use std::str::*;

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
    Path(CanvasPath),

    /// Group of other shapes (with no shape itself)
    Group,

    /// Rectangle shape
    Rectangle(CanvasRectangle),

    /// Ellipse filling a rectangle
    Ellipse(CanvasEllipse),

    /// Polygon filling a rectangle, with the specified number of points
    Polygon(CanvasPolygon),
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

    ///
    /// Creates a shape ID from a string
    ///
    pub fn from_string(string_guid: &str) -> Self {
        Self(Uuid::from_str(string_guid).unwrap())
    }

    ///
    /// Returns the string representation of this shape ID
    ///
    #[inline]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

pub const CANVAS_GROUP_V1_TYPE: i64 = 4;
