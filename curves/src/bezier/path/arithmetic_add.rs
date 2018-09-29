use super::path::*;
use super::graph_path::*;
use super::is_clockwise::*;
use super::super::curve::*;
use super::super::super::geo::*;
use super::super::super::coordinate::*;

/// Source of a path in the graphpath
#[derive(Copy, Clone)]
enum SourcePath {
    Path1,
    Path2
}

/// Target of a path in the graphpath
#[derive(Copy, Clone)]
enum PathDirection {
    Clockwise,
    Anticlockwise
}

impl<'a, P: BezierPath> From<&'a P> for PathDirection
where P::Point: Coordinate2D {
    #[inline]
    fn from(path: &'a P) -> PathDirection {
        if path.is_clockwise() {
            PathDirection::Clockwise
        } else {
            PathDirection::Anticlockwise
        }
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
    // TODO: this doesn't add any 'holes'
    let mut merged_path = GraphPath::new();
    let mut bounds      = Bounds::empty();
    
    merged_path = merged_path.merge(GraphPath::from_path(&path1[0], (SourcePath::Path1, PathDirection::from(&path1[0]))));
    bounds      = bounds.union_bounds(path1[0].fast_bounding_box());

    // Collide with the target side to generate a full path
    // TODO: this doesn't add any 'holes'
    merged_path = merged_path.collide(GraphPath::from_path(&path2[0], (SourcePath::Path2, PathDirection::from(&path2[0]))), accuracy);
    bounds      = bounds.union_bounds(path1[0].fast_bounding_box());

    // Cast a line from a point known to be on the outside to discover an edge on the outside
    let outside_point   = bounds.min() - Point::from_components(&[0.1, 0.1]);
    let point_on_curve  = merged_path.edges_for_point(0).nth(0).unwrap().start_point();
    let outer_edge      = merged_path.line_collision(&(outside_point, point_on_curve)).unwrap().0.into();

    // Trace the external edges from the path to find the exterior edges
    merged_path.classify_exterior_edges(outer_edge, |graph, start_edge, edge_choices| {
        edge_choices.into_iter().nth(0).unwrap().into()
     });

    unimplemented!()
}
