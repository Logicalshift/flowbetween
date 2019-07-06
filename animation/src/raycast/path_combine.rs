use flo_curves::*;
use flo_curves::bezier::path::*;

///
/// Computes the bounding box of a set of paths
///
#[inline]
fn get_bounds<P: BezierPath>(path: &Vec<P>) -> Bounds<P::Point> {
    path.iter()
        .map(|path| path.bounding_box::<Bounds<_>>())
        .fold(Bounds::empty(), |a, b| a.union_bounds(b))
}

///
/// If two paths can be combined, generates the 
///
pub fn combine_paths<P: BezierPath>(path1: &Vec<P>, path2: &Vec<P>, accuracy: f64) -> Option<GraphPath<P::Point, PathLabel>>
where P::Point: Coordinate2D {
    // Nothing to combine if h
    if path1.len() == 0 || path2.len() == 0 {
        return None;
    }

    // Paths do not overlap if their bounding boxes do not overlap
    let bounds1 = get_bounds(path1);
    let bounds2 = get_bounds(path2);

    if !bounds1.overlaps(&bounds2) {
        // Neither of the paths overlap as the bounding boxes are different
        None
    } else {
        // Convert both to graph paths
        let path1 = GraphPath::from_merged_paths(path1.into_iter().map(|path| (path, PathLabel(0, PathDirection::from(path)))));
        let path2 = GraphPath::from_merged_paths(path2.into_iter().map(|path| (path, PathLabel(1, PathDirection::from(path)))));

        match path1.collide_or_merge(path2, accuracy) {
            CollidedGraphPath::Collided(collided_path)  => Some(collided_path),
            CollidedGraphPath::Merged(merged_path)      => None                     // TODO: except if path1/path2 is entirely enclosed in the other 
        }
    }
}
