use crate::traits::*;
use crate::raycast::edge::*;
use super::keyframe_core::*;

use flo_curves::bezier::*;
use flo_curves::bezier::path::algorithms::*;

use std::sync::*;
use std::cmp::{Ordering};
use std::time::{Duration};

///
/// Represents a collision along a raycast edge
///
struct VectorCollision {
    pos:        PathPoint,
    line_t:     f64,
    element_id: ElementId
}

impl Ord for VectorCollision {
    fn cmp(&self, b: &VectorCollision) -> Ordering {
        if self.line_t < b.line_t {
            Ordering::Less
        } else if self.line_t > b.line_t {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for VectorCollision {
    fn partial_cmp(&self, b: &VectorCollision) -> Option<Ordering> {
        Some(self.cmp(b))
    }

}

impl PartialEq for VectorCollision {
    fn eq(&self, b: &VectorCollision) -> bool{
        self.line_t == b.line_t
    }
}

impl Eq for VectorCollision {

}

impl KeyFrameCore {
    ///
    /// Retrieves a ray-casting function for a particular frame
    ///
    /// The function that this returns will determine where a ray intersects the vector objects in the frame.
    ///
    pub (super) fn raycast<'a>(&'a self, when: Duration) -> impl 'a+Fn(PathPoint, PathPoint) -> Vec<RayCollision<PathPoint, ElementId>> {
        // Collect all of the vector elements in the frame into a single place
        // If this isn't a vector frame, we'll use the empty list
        let all_elements = self.vector_elements(when);

        // Convert the elements into edges
        let mut edges       = vec![];
        for element in all_elements {
            let properties = self.apply_properties_for_element(&element, Arc::new(VectorProperties::default()), when);
            edges.extend(RaycastEdge::from_vector(&element, Arc::clone(&properties)));
        }

        // Generate the final function
        move |from, to| {
            let ray = (from, to);

            // Cast the ray against all edges (simplest algorithm, but slowest too)
            let collisions = edges.iter()
                .flat_map(|edge| curve_intersects_ray(&edge.curve, &ray)
                    .into_iter()
                    .map(move |(_curve_t, line_t, pos)| VectorCollision { line_t: line_t, pos: pos, element_id: edge.element_id }));

            // Collect into an ordered list
            let mut collisions = collisions.collect::<Vec<_>>();
            if collisions.len() > 0 {
                // TODO: use the sorted collisions to hide any edge that is underneath an edge that comes from an eraser
                collisions.sort();

                collisions.into_iter()
                    .map(|collision| RayCollision::new(collision.pos, collision.element_id))
                    .collect()
            } else {
                // Short-circuit the case where there are no collisions
                vec![]
            }
       }
    }
}