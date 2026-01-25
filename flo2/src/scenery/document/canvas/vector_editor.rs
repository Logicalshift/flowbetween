use super::layer::*;
use super::property::*;
use super::shape::*;

use flo_scene::*;

use ::serde::*;

///
/// Basic editing actions for a vector canvas
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
pub enum VectorCanvas {
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

    /// Adds properties to a layer
    SetLayerProperties(CanvasLayerId, Vec<(CanvasPropertyId, CanvasProperty)>),

    /// Adds properties to a shape
    SetShapeProperties(CanvasShapeId, Vec<(CanvasPropertyId, CanvasProperty)>),

    /// Unsets properties for a layer if they're already set
    RemoveLayerProperties(CanvasLayerId, Vec<CanvasPropertyId>),

    /// Unsets properties for a shape if they're already set
    RemoveShapeProperties(CanvasShapeId, Vec<CanvasPropertyId>),

    /// Subscribe for any updates to this canvas (eg, to implement a rendering program)
    Subscribe(StreamTarget),
}

///
/// Message sent to subprograms that subscribe to vector canvas updates
///
/// The vector canvas provides a stream of notifications of the IDs of the things that have changed but not the
/// actual changes themselves
///
#[derive(Clone, Serialize, Deserialize)]
pub enum VectorCanvasUpdate {
    /// Indicates that the specified layers have been changed (had shapes added or removed, properties changes or have been added or deleted)
    LayerChanged(Vec<CanvasLayerId>),

    /// Indicates that the specified shape has been changed (added or deleted, properties or attached shapes changed)
    ShapeChanged(Vec<CanvasShapeId>),
}

impl SceneMessage for VectorCanvas {

}

impl SceneMessage for VectorCanvasUpdate {

}
