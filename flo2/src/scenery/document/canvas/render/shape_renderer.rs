use super::shape_type_renderer::*;
use super::super::frame_time::*;

use flo_draw::canvas::*;
use flo_scene::*;
use flo_scene::commands::*;

use futures::prelude::*;
use futures::stream::{FuturesUnordered};

use std::collections::*;
use std::sync::*;

///
/// Requests the rendering instructions for a set of shapes
///
/// The returned list is in the same order as the original iterator, and will always have the same number of entries.
///
pub async fn render_shapes(shapes: impl Iterator<Item=Arc<ShapeWithProperties>>, frame_time: FrameTime, context: &SceneContext) -> Vec<Arc<Vec<Draw>>> {
    // Sort the shapes into bins by target program ID, and also remember the order that we need to read values from the bins (so we make one request per program regardless of the ordering of the shapes)
    let mut shape_bins = HashMap::new();
    let mut read_order = vec![];

    for shape in shapes {
        let shape_type = shape.shape_type;

        // Bin this shape for the program ID
        shape_bins.entry(shape_type)
            .or_insert_with(|| vec![])
            .push(shape);
        read_order.push(shape_type);
    }

    // Query all of the programs to build up the result
    let mut requests = shape_bins.into_iter()
        .map(|(shape_type, shapes)| async move {
            let num_shapes      = shapes.len();
            let query_rendering = context.spawn_query(ReadCommand::default(), RenderShapesRequest::RenderRequest(shapes, frame_time, ().into()), shape_type.render_program_id());

            if let Ok(query_rendering) = query_rendering {
                (shape_type, query_rendering.collect::<Vec<_>>().await)
            } else {
                (shape_type, vec![RenderShapesResponse::ShapeRendering(Arc::new(vec![])); num_shapes])
            }
        })
        .collect::<FuturesUnordered<_>>();

    // Wait for the requests to generate their sets of shapes
    let mut generated_drawing = HashMap::new();
    while let Some((shape_type, drawings)) = requests.next().await {
        generated_drawing.insert(shape_type, drawings.into_iter());
    }

    // Use the read_order to read from the resulting vecs to generate the final result
    let mut result = vec![];
    for shape_type in read_order {
        if let Some(drawing) = generated_drawing.get_mut(&shape_type).and_then(|shape_iter| shape_iter.next()) {
            match drawing {
                RenderShapesResponse::ShapeRendering(drawing) => {
                    result.push(drawing);
                }
            }
        }
    }

    result
}
