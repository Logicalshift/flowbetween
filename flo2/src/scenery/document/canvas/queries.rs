use super::brush::*;
use super::frame_time::*;
use super::layer::*;
use super::property::*;
use super::shape::*;
use super::shape_type::*;

use flo_scene::*;
use flo_scene::commands::*;
use flo_scene::programs::*;
use futures::prelude::*;
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
    Shape(CanvasShapeId, CanvasShape, FrameTime, ShapeType, Vec<(CanvasPropertyId, CanvasProperty)>),

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

///
/// Queries the whole vector document's state at the given time
///
#[inline]
pub fn query_vector_whole_document(when: FrameTime) -> impl Stream<Item=VectorResponse> {
    let context = scene_context().unwrap();

    context.spawn_query(ReadCommand::default(), VectorQuery::WholeDocument(().into(), when), ()).unwrap()
}

///
/// Queries the outline of the document (document properties, which layers are present)
///
#[inline]
pub fn query_vector_outline() -> impl Stream<Item=VectorResponse> {
    let context = scene_context().unwrap();

    context.spawn_query(ReadCommand::default(), VectorQuery::DocumentOutline(().into()), ()).unwrap()
}

///
/// Queries the contents of a set of layers at the given time
///
#[inline]
pub fn query_vector_layers(when: FrameTime, layers: impl IntoIterator<Item=CanvasLayerId>) -> impl Stream<Item=VectorResponse> {
    let context = scene_context().unwrap();

    context.spawn_query(ReadCommand::default(), VectorQuery::Layers(().into(), layers.into_iter().collect(), when), ()).unwrap()
}

///
/// Queries the values for a set of shapes
///
#[inline]
pub fn query_vector_shapes(shapes: impl IntoIterator<Item=CanvasShapeId>) -> impl Stream<Item=VectorResponse> {
    let context = scene_context().unwrap();

    context.spawn_query(ReadCommand::default(), VectorQuery::Shapes(().into(), shapes.into_iter().collect()), ()).unwrap()
}

///
/// Queries the values for a set of brushes (brushes are used to set properties on many shapes at once without having to duplicate them)
///
#[inline]
pub fn query_vector_brushes(brushes: impl IntoIterator<Item=CanvasBrushId>) -> impl Stream<Item=VectorResponse> {
    let context = scene_context().unwrap();

    context.spawn_query(ReadCommand::default(), VectorQuery::Brushes(().into(), brushes.into_iter().collect()), ()).unwrap()
}
