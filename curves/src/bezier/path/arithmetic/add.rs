use super::arithmetic::*;
use super::super::path::*;
use super::super::graph_path::*;
use super::super::super::super::coordinate::*;

//
// This uses a simple ray casting algorithm to perform the addition
// 
// Basic idea is to cast a ray at an edge which is currently uncategorised, and mark the edges it crosses as interior or
// exterior depending on whether or not we consider it as crossing into or out of the final shape.
//

impl<Point: Coordinate+Coordinate2D> GraphPath<Point, PathLabel> {
    ///
    /// Given a labelled graph path, marks exterior edges by adding `PathSource::Path1` and `PathSource::Path2`
    ///
    pub fn set_exterior_by_adding(&mut self) {
        // Use an even-odd winding rule (all edges are considered 'external')
        self.set_edge_kinds_by_ray_casting(|path1_crossings, path2_crossings| (path1_crossings&1) != 0 || (path2_crossings&1) != 0);
    }

    ///
    /// Given a path that intersects itself (ie, only contains SourcePath::Path1), discovers the 'true' exterior edge.
    ///
    pub fn set_exterior_by_removing_interior_points(&mut self) {
        // All points inside the path are considered 'interior' (non-zero winding rule)
        self.set_edge_kinds_by_ray_casting(|path1_crossings, path2_crossings| path1_crossings != 0 || path2_crossings != 0);
    }
}

///
/// Generates the path formed by adding two sets of paths
/// 
/// The input vectors represent the external edges of the path to add (a single BezierPath cannot have any holes in it, so a set of them
/// effectively represents a path intended to be rendered with an even-odd winding rule)
///
pub fn path_add<Point, P1: BezierPath<Point=Point>, P2: BezierPath<Point=Point>, POut: BezierPathFactory<Point=Point>>(path1: &Vec<P1>, path2: &Vec<P2>, accuracy: f64) -> Vec<POut>
where   Point: Coordinate+Coordinate2D {
    // If either path is empty, short-circuit by returning the other
    if path1.len() == 0 {
        return path2.iter()
            .map(|path| POut::from_path(path))
            .collect();
    } else if path2.len() == 0 {
        return path1.iter()
            .map(|path| POut::from_path(path))
            .collect();
    }

    // Create the graph path from the source side
    let mut merged_path = GraphPath::new();
    merged_path         = merged_path.merge(GraphPath::from_merged_paths(path1.into_iter().map(|path| (path, PathLabel(PathSource::Path1, PathDirection::from(path))))));

    // Collide with the target side to generate a full path
    merged_path         = merged_path.collide(GraphPath::from_merged_paths(path2.into_iter().map(|path| (path, PathLabel(PathSource::Path2, PathDirection::from(path))))), accuracy);

    // Set the exterior edges using the 'add' algorithm
    merged_path.set_exterior_by_adding();
    merged_path.heal_exterior_gaps();

    // Produce the final result
    merged_path.exterior_paths()
}

///
/// Generates the path formed by removing any interior points from an existing path
///
pub fn path_remove_interior_points<Point, P1: BezierPath<Point=Point>, POut: BezierPathFactory<Point=Point>>(path: &Vec<P1>, accuracy: f64) -> Vec<POut>
where   Point: Coordinate+Coordinate2D {
    // Create the graph path from the source side
    let mut merged_path = GraphPath::new();
    merged_path         = merged_path.merge(GraphPath::from_merged_paths(path.into_iter().map(|path| (path, PathLabel(PathSource::Path1, PathDirection::from(path))))));

    // Collide the path with itself to find the intersections
    merged_path.self_collide(accuracy);

    // Set the exterior edges using the 'add' algorithm
    merged_path.set_exterior_by_removing_interior_points();
    merged_path.heal_exterior_gaps();

    // Produce the final result
    merged_path.exterior_paths()
}
