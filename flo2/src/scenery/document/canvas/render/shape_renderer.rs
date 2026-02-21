use super::shape_type_renderer::*;

use flo_draw::canvas::*;
use flo_scene::*;
use flo_scene::commands::*;

use futures::prelude::*;

use std::collections::*;
use std::sync::*;

///
/// Requests the rendering instructions for a set of shapes
///
/// The returned list is in the same order as the original iterator, and will always have the same number of entries.
///
pub async fn render_shapes(shapes: impl Iterator<Item=Arc<ShapeWithProperties>>, context: &SceneContext) -> Vec<Arc<Vec<Draw>>> {
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
    let requests = shape_bins.into_iter()
        .map(|(shape_type, shapes)| async move {
            let num_shapes      = shapes.len();
            let query_rendering = context.spawn_query(ReadCommand::default(), RenderShapesRequest::RenderRequest(shapes, ().into()), shape_type.render_program_id());

            if let Ok(query_rendering) = query_rendering {
                (shape_type, query_rendering.collect::<Vec<_>>().await)
            } else {
                (shape_type, vec![RenderShapesResponse::ShapeRendering(Arc::new(vec![])); num_shapes])
            }
        });

    // TODO: wait for the requests to generate their sets of shapes
    // TODO: use the read_order to read from the resulting vecs to generate the final result

    // Build the result
    let mut result = vec![];
    result
}
