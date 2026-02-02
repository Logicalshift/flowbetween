use super::brush::*;
use super::layer::*;
use super::property::*;
use super::shape::*;

use crate::scenery::ui::*;

use flo_scene::*;
use serde::*;

///
/// Queries that can be made on a vector document
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VectorQuery {
    /// Queries all of the entities in the document, sending a response as a `QueryResponse<VectorResponse>`
    WholeDocument(StreamTarget),

    /// Queries the document and layer properties without returning any shape data
    DocumentOutline(StreamTarget),

    /// Queries the entities associated with the specified layer
    Layers(StreamTarget, Vec<CanvasLayerId>),

    /// Queries specific shapes
    Shapes(StreamTarget, Vec<CanvasShapeId>),

    /// Queries the properties of the specified set of brushes
    Brushes(StreamTarget, Vec<CanvasBrushId>),

    /// Queries the shapes that can be found in a particular region (in a range of layers). If 'inclusive' is true then the shapes must lie entirely in the specified region.
    ShapesInRegion { target: StreamTarget, search_layers: Vec<CanvasLayerId>, region: (UiPoint, UiPoint), inclusive: bool },

    /// Queries the shapes that can be found at a particular point
    ShapesAtPoint { target: StreamTarget, search_layers: Vec<CanvasLayerId>, point: UiPoint },
}

///
/// The responses from a vector query 
///
#[derive(Serialize, Deserialize, Clone)]
pub enum VectorResponse {
    /// Specifies the properties for the document
    Document(Vec<(CanvasPropertyId, CanvasProperty)>),

    /// Specifies the layers defined in this document, in the order that they're rendered in
    LayerOrder(Vec<CanvasLayerId>),

    /// Indicates the properties associated with a brush
    Brush(CanvasBrushId, Vec<(CanvasPropertyId, CanvasProperty)>),

    /// The following items are part of the specified layer
    Layer(CanvasLayerId, Vec<(CanvasPropertyId, CanvasProperty)>),

    /// Indicates the definition of a shape. These are returned in bottom-to-top order. Properties from brushes are 
    /// already added to the shape, but the attached brush IDs are returned in case they're useful
    Shape(CanvasShapeId, Vec<CanvasBrushId>, Vec<(CanvasPropertyId, CanvasProperty)>),

    /// The following shapes are parented to the previous shape
    StartGroup,

    /// Ends a group started with 'StartGroup' (so any following shapes are parented to the layer or shape that was
    /// being generated before)
    EndGroup,
}

impl SceneMessage for VectorQuery {

}

impl SceneMessage for VectorResponse {

}