use super::super::queries::*;

use flo_scene::*;
use flo_draw::canvas::*;

///
/// Renders the shapes on a layer when described as a set of vector responses
///
pub async fn render_layer(layer: impl Send + IntoIterator<Item=&VectorResponse>, context: &SceneContext) -> Vec<Draw> {
    // The drawing instructions for this layer
    let mut drawing = vec![];

    // TODO: need a cache of connections for shapes (probably shared in a type that's passed in)

    // TODO: rendering needs to render as many shapes at once as it can (for perf reasons), but needs multiple passes for groups
    // TODO: so, we find the 'deepest' nodes first, then progresively render the higher nodes when they become available

    drawing
}
