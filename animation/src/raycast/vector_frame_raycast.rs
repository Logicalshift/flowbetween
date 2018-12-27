use super::edge::*;
use super::super::traits::*;

use std::sync::*;

///
/// Retrieves a ray-casting function for a particular frame
/// 
/// The function that this returns will determine where a ray intersects the vector objects in the frame.
///
pub fn vector_frame_raycast<'a, FrameType: Frame>(frame: &'a FrameType) -> impl 'a+Fn(PathPoint, PathPoint) -> Vec<PathPoint> {
    // Collect all of the vector elements in the frame into a single place
    // If this isn't a vector frame, we'll use the empty list
    let all_elements = frame.vector_elements()
        .unwrap_or_else(|| Box::new(vec![].into_iter()));

    // Convert the elements into edges
    let mut edges       = vec![];
    let mut properties  = Arc::new(VectorProperties::default());

    for element in all_elements {
        properties = element.update_properties(properties);
        edges.extend(RaycastEdge::from_vector(&element, Arc::clone(&properties)));
    }

    // Generate the final function
    move |from, to| {
        vec![]
    }
}
