use super::shape_renderer::*;
use super::shape_type_renderer::*;
use super::super::frame_time::*;
use super::super::queries::*;

use flo_scene::*;
use flo_draw::canvas::*;

use std::sync::*;

///
/// Renders the shapes on a layer when described as a set of vector responses
///
pub async fn render_layer(layer: impl Send + IntoIterator<Item=VectorResponse>, frame_time: FrameTime, context: &SceneContext) -> Vec<Draw> {
    struct RenderItem {
        /// Shape to render
        shape:          Arc<ShapeWithProperties>,

        /// Child nodes for the shape if it's a group (must be rendered before this shape can be rendered)
        child_nodes:    Vec<usize>,

        /// The rendering for this shape, once it's complete
        drawing:        Option<Arc<Vec<Draw>>>,

        /// Height of this shape in the tree (distance from the top of the tree)
        height:         usize,
    }

    // Build up a representation of the items we're going to render (discover the children of group nodes, and the height each node is in the tree)
    let mut render: Vec<RenderItem> = vec![];
    let mut parent_stack            = vec![];
    let mut root_nodes              = vec![];
    let mut max_height              = 0;

    for response in layer {
        match response {
            VectorResponse::StartGroup => {
                // The following items are parented to this item
                parent_stack.push(render.len()-1);

                // Update the height of all of the parent nodes
                for (height, parent_idx) in parent_stack.iter().rev().enumerate() {
                    if render[*parent_idx].height == height + 1 { break; }

                    render[*parent_idx].height = height + 1;
                }

                // Update the maximum height of the render stack
                max_height = parent_stack.len().max(max_height);
            }

            VectorResponse::EndGroup => {
                // This item is not parented
                parent_stack.pop();
            }

            VectorResponse::Shape(_shape_id, shape, shape_time, shape_type, properties) => {
                // Create a node for this shape
                let node_idx    = render.len();
                let parent_idx  = parent_stack.last().copied();
                let properties  = Arc::new(properties);
                let group       = vec![];

                // Everything starts out at height 0
                render.push(RenderItem {
                    shape:          Arc::new(ShapeWithProperties { shape, shape_type, shape_time, properties, group }),
                    child_nodes:    vec![],
                    drawing:        None,
                    height:         0,
                });

                // Add as a child node to the current parent
                if let Some(parent_idx) = parent_idx {
                    let new_idx = render.len() - 1;
                    render[parent_idx].child_nodes.push(new_idx);
                } else {
                    root_nodes.push(node_idx);
                }
            }


            // Ignore everything else
            _ => {}
        }
    }

    // Bin into render passes. By rendering down from the leaves, we ensure that the drawing instructions needed to render each group are available just in time
    // By processing the whole 'level' in the render tree all in one go, we minimize the number of calls to `render_shapes`
    let mut render_passes = vec![vec![]; max_height+1];
    for (idx, item) in render.iter().enumerate() {
        render_passes[item.height].push(idx);
    }

    // Perform each render pass to build up the drawing instructions for this layer
    for render_pass_idxs in render_passes.into_iter() {
        // Fill in the drawing instructions for any child items
        for render_idx in render_pass_idxs.iter() {
            let render_idx = *render_idx;

            // Short-circuit if the item isn't a group item
            if render[render_idx].child_nodes.is_empty() { continue; }

            // Gather the children for this shape
            let children = render[render_idx].child_nodes.iter()
                .map(|child_idx| (render[*child_idx].shape.clone(), render[*child_idx].drawing.clone().unwrap()))
                .collect::<Vec<_>>();

            // Modify the shape to contain the items in its group
            // TODO: Rust doesn't want the shape to be invalid even when we're putting it right back, so we end up cloning it, 
            // would be more efficient to let the field be invalid before overwriting it instead ('safe' if Rust knows that 
            // when we write it back we're not dropping the original)
            let item        = &mut render[render_idx];
            let mut shape   = Arc::unwrap_or_clone(item.shape.clone());

            shape.group = children;

            item.shape  = Arc::new(shape);
        }

        // Render the shapes in this pass
        let drawings = render_shapes(render_pass_idxs.iter().map(|idx| Arc::clone(&render[*idx].shape)), frame_time, context).await;

        // Put the drawings back into the render items
        for (idx, drawing) in render_pass_idxs.into_iter().zip(drawings) {
            render[idx].drawing = Some(drawing);
        }
    }

    // The drawing instructions for this layer are now just the root nodes concatenated together
    let mut drawing = vec![];

    for node_idx in root_nodes {
        drawing.extend(render[node_idx].drawing.as_ref().unwrap().iter().cloned());
    }

    drawing
}
