use super::super::property::*;
use super::super::shape::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::canvas::*;
use ::serde::*;

use std::sync::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShapeWithProperties {
    /// The shape definition
    pub shape: CanvasShape,

    /// The properties for the shape
    pub properties: Vec<(CanvasPropertyId, CanvasProperty)>,
}

///
/// A render request changes some shape data into the corresponding drawing instructions to render that shape
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RenderRequest {
    /// Queries the rendering instructions for each shape in the list. The query should return one response per shape
    RenderRequest(Vec<Arc<ShapeWithProperties>>, StreamTarget),
}

///
/// Response to a render request
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RenderResponse {
    /// Rendering instructions for the shapes in the render request (same number and in the same order they were requested in)
    ShapeRendering(Vec<Arc<Vec<Draw>>>)
}

impl SceneMessage for RenderRequest {
}

impl QueryRequest for RenderRequest {
    type ResponseData = RenderResponse;

    fn with_new_target(self, new_target: StreamTarget) -> Self {
        match self {
            RenderRequest::RenderRequest(shapes, _old_target) => RenderRequest::RenderRequest(shapes, new_target)
        }
    }
}
