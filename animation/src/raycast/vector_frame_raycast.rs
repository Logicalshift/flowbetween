use super::edge::*;
use super::super::traits::*;

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

    // TODO: Convert the elements into edges
    

    // Generate the final function
    move |from, to| {
        vec![]
    }
}
