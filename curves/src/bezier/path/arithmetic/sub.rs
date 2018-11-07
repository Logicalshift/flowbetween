use super::arithmetic::*;
use super::super::path::*;
use super::super::graph_path::*;
use super::super::super::super::coordinate::*;

impl<Point: Coordinate+Coordinate2D> GraphPath<Point, PathLabel> {
    ///
    /// Given a labelled graph path, marks exterior edges by subtracting `PathSource::Path1` and `PathSource::Path2`
    ///
    pub fn set_exterior_by_subtracting(&mut self) {
        // Use an even-odd winding rule (all edges are considered 'external')
        self.set_edge_kinds_by_ray_casting(|path1_crossings, path2_crossings| (path1_crossings&1) != 0 && (path2_crossings&1) == 0);
    }
}

///
/// Generates the path formed by subtracting two sets of paths
/// 
/// The input vectors represent the external edges of the path to subtract (a single BezierPath cannot have any holes in it, so a set of them
/// effectively represents a path intended to be rendered with an even-odd winding rule)
///
pub fn path_sub<Point, P1: BezierPath<Point=Point>, P2: BezierPath<Point=Point>, POut: BezierPathFactory<Point=Point>>(path1: &Vec<P1>, path2: &Vec<P2>, accuracy: f64) -> Vec<POut>
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

    // Set the exterior edges using the 'subtract' algorithm
    merged_path.set_exterior_by_subtracting();
    merged_path.heal_exterior_gaps();

    // Produce the final result
    merged_path.exterior_paths()
}
