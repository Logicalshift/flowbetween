use super::brush::*;
use super::frame_time::*;
use super::layer::*;
use super::property::*;
use super::shape::*;
use super::shape_type::*;

use flo_scene::*;
use flo_scene::programs::*;
use serde::*;

///
/// Queries that can be made on a vector document
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VectorQuery {
    /// Queries all of the entities in the document for a frame at the specified time, sending a response as a `QueryResponse<VectorResponse>`
    WholeDocument(StreamTarget, FrameTime),

    /// Queries the document and layer properties without returning any shape data
    DocumentOutline(StreamTarget),

    /// Queries the entities associated with the specified layers for a frame at the specified time
    Layers(StreamTarget, Vec<CanvasLayerId>, FrameTime),

    /// Queries specific shapes
    Shapes(StreamTarget, Vec<CanvasShapeId>),

    /// Queries the properties of the specified set of brushes
    Brushes(StreamTarget, Vec<CanvasBrushId>),
}

///
/// The responses from a vector query 
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum VectorResponse {
    /// Specifies the properties for the document
    Document(Vec<(CanvasPropertyId, CanvasProperty)>),

    /// Specifies the layers defined in this document, in the order that they're rendered in
    LayerOrder(Vec<CanvasLayerId>),

    /// Indicates the properties associated with a brush
    Brush(CanvasBrushId, Vec<(CanvasPropertyId, CanvasProperty)>),

    /// The following items are part of the specified layer
    Layer(CanvasLayerId, Vec<(CanvasPropertyId, CanvasProperty)>),

    /// A frame starts at the specified point in time (on the layer that was previously indicated with `Layer`)
    Frame(FrameTime),

    /// Indicates the definition of a shape. These are returned in bottom-to-top order. Properties come from the
    /// shape itself, along with any attached brushes
    Shape(CanvasShapeId, CanvasShape, ShapeType, Vec<(CanvasPropertyId, CanvasProperty)>),

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

impl QueryRequest for VectorQuery {
    type ResponseData = VectorResponse;

    fn with_new_target(self, new_target: StreamTarget) -> Self {
        use VectorQuery::*;

        match self {
            WholeDocument(_target, when)    => WholeDocument(new_target, when),
            DocumentOutline(_target)        => DocumentOutline(new_target),
            Layers(_target, layers, when)   => Layers(new_target, layers, when),
            Shapes(_target, shape_id)       => Shapes(new_target, shape_id),
            Brushes(_target, brush_id)      => Brushes(new_target, brush_id),
        }
    }
}
