use super::super::property::*;
use super::super::shape::*;

use futures::prelude::*;

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
    pub properties: Arc<Vec<(CanvasPropertyId, CanvasProperty)>>,

    /// The shapes that are grouped under this one, with their drawing instructions
    pub group: Vec<(Arc<ShapeWithProperties>, Arc<Vec<Draw>>)>,
}

///
/// A render request changes some shape data into the corresponding drawing instructions to render that shape
///
/// This is the request made to the subprogram for a shapetype to render that specific shape
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RenderShapesRequest {
    /// Queries the rendering instructions for each shape in the list. The query should return one response per shape
    RenderRequest(Vec<Arc<ShapeWithProperties>>, StreamTarget),
}

///
/// Response to a render request
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RenderShapesResponse {
    /// Rendering instructions for the shapes in the render request (same number and in the same order they were requested in)
    ShapeRendering(Arc<Vec<Draw>>)
}

impl SceneMessage for RenderShapesRequest {
}

impl QueryRequest for RenderShapesRequest {
    type ResponseData = RenderShapesResponse;

    fn with_new_target(self, new_target: StreamTarget) -> Self {
        match self {
            Self::RenderRequest(shapes, _old_target) => Self::RenderRequest(shapes, new_target)
        }
    }
}

impl SceneMessage for RenderShapesResponse {
}

///
/// Runs a shape renderer program that uses the supplied function to generate the drawing instructions for a shape
///
pub async fn shape_renderer_program(input: InputStream<RenderShapesRequest>, context: SceneContext, shape_renderer: impl 'static + Send + Sync + Fn(&ShapeWithProperties, &mut Vec<Draw>)) {
    let mut input       = input;
    let shape_renderer  = Arc::new(shape_renderer);

    while let Some(request) = input.next().await {
        match request {
            RenderShapesRequest::RenderRequest(shapes, response_target) => {
                // Send to the target (we'll just ignore errors by not doing any work)
                let Ok(mut target) = context.send(response_target) else { continue; };

                // Send a query response
                let shape_renderer = Arc::clone(&shape_renderer);
                target.send(QueryResponse::with_iterator(shapes.into_iter().map(move |shape| {
                    let mut drawing = vec![];
                    (shape_renderer)(&*shape, &mut drawing);

                    RenderShapesResponse::ShapeRendering(Arc::new(drawing))
                }))).await.ok();
            }
        }
    }
}
