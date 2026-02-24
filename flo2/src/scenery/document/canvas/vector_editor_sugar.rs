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
/// Adds a shape to the canvas in the current scene
///
pub async fn vector_add_shape(shape_type: impl Into<ShapeType>, shape: impl Into<CanvasShape>, parent: impl Into<CanvasShapeParent>, properties: &impl ToCanvasProperties) -> CanvasShapeId {
    // Fetch the context
    let context             = scene_context().expect("Must be called from a flo_scene subprogram");
    let mut vector_editor   = context.send(()).unwrap();

    // Create a shape ID and add it to the canvas
    let shape_id = CanvasShapeId::new();

    vector_editor.send(VectorCanvas::AddShape(shape_id, shape_type.into(), shape.into())).await.unwrap();
    vector_editor.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Shape(shape_id), properties.to_properties())).await.unwrap();
    vector_editor.send(VectorCanvas::SetShapeParent(shape_id, parent.into())).await.unwrap();

    shape_id
}
