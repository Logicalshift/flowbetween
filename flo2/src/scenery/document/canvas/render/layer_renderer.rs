use super::shape_type_renderer::*;
use super::super::queries::*;

use flo_scene::*;
use flo_draw::canvas::*;

use std::sync::*;

///
/// Renders the shapes on a layer when described as a set of vector responses
///
pub async fn render_layer(layer: impl Send + IntoIterator<Item=VectorResponse>, context: &SceneContext) -> Vec<Draw> {
    struct RenderItem {
        shape:          Arc<ShapeWithProperties>,
        child_nodes:    Vec<usize>,
        parent_node:    Option<usize>,
        render:         Option<Arc<Draw>>,
    }

    // Rendering involves multiple passes over the layer
    let mut render          = vec![];
    let mut parent_stack    = vec![];

    for response in layer {
        match response {
            VectorResponse::StartGroup => {
                // The following items are parented to this item
                parent_stack.push(render.len()-1);
            }

            VectorResponse::EndGroup => {
                // This item is not parented
                parent_stack.pop();
            }

            VectorResponse::Shape(_shape_id, shape, shape_type, properties) => {
                // Create a node for this shape
                let parent_idx  = parent_stack.last().copied();
                let properties  = Arc::new(properties);
                let group       = vec![];

                render.push(RenderItem {
                    shape:          Arc::new(ShapeWithProperties { shape, shape_type, properties, group }),
                    child_nodes:    vec![],
                    parent_node:    parent_idx,
                    render:         None,
                });

                // Add as a child node to the current parent
                if let Some(parent_idx) = parent_idx {
                    let new_idx = render.len() - 1;
                    render[parent_idx].child_nodes.push(new_idx);
                }
            }


            // Ignore everything else
            _ => {}
        }
    }

    enum RenderOp {
        /// Render the shape at the specified index, storing in the render queue
        RenderShape(usize),

        /// Pop a result from the front of the render queue and push to the list of final rendering instructions
        PushResult,

        /// Pop a result from the front of the render queue and push it to the list of child nodes for the specified item
        AddChild(usize),
    }

    // TODO: rendering needs to render as many shapes at once as it can (for perf reasons), but needs multiple passes for groups
    // TODO: so, we find the 'deepest' nodes first, then progresively render the higher nodes when they become available

    // The drawing instructions for this layer
    let mut drawing = vec![];

    drawing
}
