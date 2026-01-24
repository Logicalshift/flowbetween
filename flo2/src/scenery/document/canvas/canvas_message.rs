use crate::scenery::ui::*;

use flo_draw::canvas::*;
use flo_scene::*;

use ::serde::*;
use uuid::*;

use std::sync::*;

///
/// Identifier used for a layer in the canvas document
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasLayerId(Uuid);

impl CanvasLayerId {
    ///
    /// Creates a unique new canvas layer ID
    ///
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

///
/// Identifier used for a shape in the canvas document
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasShapeId(Uuid);

impl CanvasShapeId {
    ///
    /// Creates a unique new canvas path ID
    ///
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

///
/// Identifier used for a brush in the canvas document
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasBrushId(Uuid);

impl CanvasBrushId {
    ///
    /// Creates a unique new canvas brush ID
    ///
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

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

///
/// Identifier for a canvas property
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasPropertyId(usize);

///
/// Value of a specific property set on a shape, layer or brush
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub enum CanvasProperty {
    /// Property with a single float value
    Float(f64),

    /// Property with a single integer value
    Int(i64),

    /// Property with a value that's a floating point number
    FloatList(Vec<f64>),

    /// Property with a value that's a list of integers
    IntList(Vec<i64>),

    /// Property with a value that's a series of bytes
    ByteList(Vec<u8>),
}

///
/// Basic editing actions for the canvas
///
/// At a basic level, there are a few entities that can exist on a canvas:
///
///  * Layers
///  * Shapes
///  * Properties
///  * Brushes
///
/// Layers are at the top level of the document. Shapes may be attached to layers or other shapes (forming a group).
/// Properties may be attached to shapes, layers or brushes.
///
/// Properties define how a shape is drawn. Brushes form a set of properties that can be applied as a group to a shape.
///
#[derive(Clone, Serialize, Deserialize)]
pub enum CanvasEdit {
    /// Adds a layer (setting the 'before_layer' to None will create the topmost layer)
    AddLayer { new_layer_id: CanvasLayerId, before_layer: Option<CanvasLayerId>, },

    /// Removes the specified layer
    RemoveLayer(CanvasLayerId),

    /// Moves the specified layer before another layer (None to move it to the top)
    ReorderLayer { layer_id: CanvasLayerId, before_shape: Option<CanvasLayerId>, },

    /// Adds a shape with no properties (transparent fill/stroke) to the canvas. If the shape ID is already in use, this will replace the existing shape in place with the new one
    AddShape(CanvasShapeId, CanvasShape),

    /// Removes a shape from the canvas
    RemoveShape(CanvasShapeId),

    /// Moves a shape so that it appears after another shape (None detaches the shape from the canvas)
    ReorderShape { shape_id: CanvasShapeId, before_shape: Option<CanvasShapeId>, },

    /// Sets a shape as the topmost shape attached to a parent
    SetShapeParent(CanvasShapeId, CanvasShapeParent),

    /*
    /// Sets the fill colour of a path
    SetFillColor(CanvasPathId, Color),

    /// Sets the stroke colour of a path
    SetStrokeColor(CanvasPathId, Color),

    /// Sets the line width for the outline of a path
    SetLineWidth(CanvasPathId, f64),

    /// Sets the cap and join properties for a path
    SetLineProperties(CanvasPathId, LineCap, LineJoin),
    */
}

impl SceneMessage for CanvasEdit {

}