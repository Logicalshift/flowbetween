use super::brush::*;
use super::layer::*;
use super::shape::*;
use super::shape_type::*;
use super::property::*;
use super::vector_editor::*;

use flo_scene::*;

use futures::prelude::*;

impl ToCanvasProperties for &[&dyn ToCanvasProperties] {
    fn to_properties(&self) -> Vec<(CanvasPropertyId, CanvasProperty)> {
        let mut result = vec![];

        for prop in self.iter() {
            result.extend(prop.to_properties());
        }

        result
    }
}

impl ToCanvasProperties for Vec<&dyn ToCanvasProperties> {
    fn to_properties(&self) -> Vec<(CanvasPropertyId, CanvasProperty)> {
        let mut result = vec![];

        for prop in self.iter() {
            result.extend(prop.to_properties());
        }

        result
    }
}

impl ToCanvasProperties for () {
    #[inline]
    fn to_properties(&self) -> Vec<(CanvasPropertyId, CanvasProperty)> {
        vec![]
    }
}

///
/// Sets some properties on an item in the canvas
///
pub async fn vector_set_properties(target: impl Into<CanvasPropertyTarget>, properties: &impl ToCanvasProperties) {
    // Fetch the context
    let context             = scene_context().expect("Must be called from a flo_scene subprogram");
    let mut vector_editor   = context.send(()).unwrap();

    vector_editor.send(VectorCanvas::SetProperty(target.into(), properties.to_properties())).await.unwrap();
}

///
/// Removes some properties on an item in the canvas
///
pub async fn vector_remove_properties(target: impl Into<CanvasPropertyTarget>, properties: impl IntoIterator<Item=CanvasPropertyId>) {
    // Fetch the context
    let context             = scene_context().expect("Must be called from a flo_scene subprogram");
    let mut vector_editor   = context.send(()).unwrap();

    vector_editor.send(VectorCanvas::RemoveProperty(target.into(), properties.into_iter().collect())).await.unwrap();
}

///
/// Adds a new layer as the topmost layer to the canvas in the current scene
///
pub async fn vector_add_layer(properties: &impl ToCanvasProperties) -> CanvasLayerId {
    // Fetch the context
    let context             = scene_context().expect("Must be called from a flo_scene subprogram");
    let mut vector_editor   = context.send(()).unwrap();

    // Create a layer ID and add it to the canvas
    let layer_id = CanvasLayerId::new();

    vector_editor.send(VectorCanvas::AddLayer { new_layer_id: layer_id, before_layer: None } ).await.unwrap();
    vector_editor.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Layer(layer_id), properties.to_properties())).await.unwrap();

    layer_id
}

///
/// Removes a layer from the canvas in the scene
///
pub async fn vector_remove_layer(layer_id: CanvasLayerId)  {
    // Fetch the context
    let context             = scene_context().expect("Must be called from a flo_scene subprogram");
    let mut vector_editor   = context.send(()).unwrap();

    vector_editor.send(VectorCanvas::RemoveLayer(layer_id)).await.unwrap();
}

///
/// Moves a layer behind another layer in the canvas
///
pub async fn vector_order_layer(layer_id: CanvasLayerId, before_layer: Option<CanvasLayerId>)  {
    // Fetch the context
    let context             = scene_context().expect("Must be called from a flo_scene subprogram");
    let mut vector_editor   = context.send(()).unwrap();

    vector_editor.send(VectorCanvas::ReorderLayer { layer_id, before_layer }).await.unwrap();
}

///
/// Adds a shape to the canvas in the current scene
///
pub async fn vector_add_shape(shape_type: impl Into<ShapeType>, shape: impl Into<CanvasShape>, parent: impl Into<CanvasShapeParent>, properties: &impl ToCanvasProperties, brushes: impl IntoIterator<Item=CanvasBrushId>) -> CanvasShapeId {
    // Fetch the context
    let context             = scene_context().expect("Must be called from a flo_scene subprogram");
    let mut vector_editor   = context.send(()).unwrap();

    // Create a shape ID and add it to the canvas
    let shape_id = CanvasShapeId::new();

    vector_editor.send(VectorCanvas::AddShape(shape_id, shape_type.into(), shape.into())).await.unwrap();
    vector_editor.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Shape(shape_id), properties.to_properties())).await.unwrap();
    vector_editor.send(VectorCanvas::SetShapeParent(shape_id, parent.into())).await.unwrap();

    let brushes = brushes.into_iter().collect::<Vec<_>>();
    if !brushes.is_empty() {
        vector_editor.send(VectorCanvas::AddShapeBrushes(shape_id, brushes)).await.unwrap();
    }

    shape_id
}

///
/// Removes shapes from the current scene
///
pub async fn vector_remove_shapes(shape_ids: impl IntoIterator<Item=CanvasShapeId>)  {
    // Fetch the context
    let context             = scene_context().expect("Must be called from a flo_scene subprogram");
    let mut vector_editor   = context.send(()).unwrap();

    for shape_id in shape_ids {
        vector_editor.send(VectorCanvas::RemoveShape(shape_id)).await.unwrap();
    }
}

///
/// Moves a shape behind another shape in the canvas
///
pub async fn vector_order_shape(shape_id: CanvasShapeId, before_shape: Option<CanvasShapeId>)  {
    // Fetch the context
    let context             = scene_context().expect("Must be called from a flo_scene subprogram");
    let mut vector_editor   = context.send(()).unwrap();

    vector_editor.send(VectorCanvas::ReorderShape { shape_id, before_shape }).await.unwrap();
}

///
/// Adds brushes to a shape in the canvas
///
pub async fn vector_attach_brushes_to_shape(shape_id: CanvasShapeId, brushes: impl IntoIterator<Item=CanvasBrushId>) {
    // Fetch the context
    let context             = scene_context().expect("Must be called from a flo_scene subprogram");
    let mut vector_editor   = context.send(()).unwrap();

    vector_editor.send(VectorCanvas::AddShapeBrushes(shape_id, brushes.into_iter().collect())).await.unwrap();
}

///
/// Removes brushes from a shape in the canvas
///
pub async fn vector_detach_brushes_to_shape(shape_id: CanvasShapeId, brushes: impl IntoIterator<Item=CanvasBrushId>) {
    // Fetch the context
    let context             = scene_context().expect("Must be called from a flo_scene subprogram");
    let mut vector_editor   = context.send(()).unwrap();

    vector_editor.send(VectorCanvas::RemoveShapeBrushes(shape_id, brushes.into_iter().collect())).await.unwrap();
}

///
/// Adds a brush to the canvas in the current scene
///
pub async fn vector_add_brush(properties: &impl ToCanvasProperties) -> CanvasBrushId {
    // Fetch the context
    let context             = scene_context().expect("Must be called from a flo_scene subprogram");
    let mut vector_editor   = context.send(()).unwrap();

    // Create a brush ID and add it to the canvas
    let brush_id = CanvasBrushId::new();

    vector_editor.send(VectorCanvas::AddBrush(brush_id)).await.unwrap();
    vector_editor.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Brush(brush_id), properties.to_properties())).await.unwrap();

    brush_id
}

///
/// Removes brushes from the current scene
///
pub async fn vector_remove_brushes(shape_ids: impl IntoIterator<Item=CanvasBrushId>)  {
    // Fetch the context
    let context             = scene_context().expect("Must be called from a flo_scene subprogram");
    let mut vector_editor   = context.send(()).unwrap();

    for shape_id in shape_ids {
        vector_editor.send(VectorCanvas::RemoveBrush(shape_id)).await.unwrap();
    }
}
