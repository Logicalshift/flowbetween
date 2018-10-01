use super::path::*;
use super::graph_path::*;
use super::is_clockwise::*;
use super::super::curve::*;
use super::super::super::geo::*;
use super::super::super::coordinate::*;

/// Source of a path in the graphpath
#[derive(Copy, Clone, PartialEq)]
enum SourcePath {
    Path1,
    Path2
}

/// Target of a path in the graphpath
#[derive(Copy, Clone, PartialEq)]
enum PathDirection {
    Clockwise,
    Anticlockwise
}

impl PathDirection {
    #[inline]
    fn reversed(&self) -> PathDirection {
        use self::PathDirection::*;

        match self {
            Clockwise       => Anticlockwise,
            Anticlockwise   => Clockwise
        }
    }
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

//
// There are actually a couple of ways to determine which path continues on the outside edge at an intersection.
// 
// One way is to pick the path that has the shallowest angle with the incoming path (anti-clockwise or clockwise depending on
// the direction of the path containing the incoming edge)
// 
// Another way is to assume no self-intersections. This means every intersection should switch from one of the paths to the other
// and reduces the number of choices. The correct choice should always be the edge from the other path that continues moving in
// the direction we're going (ie, if we're moving clockwise, we should pick the clockwise edge)
// 
// One problem with this approach is that it's technically valid to have 'holes' with points that share a path, which may increase
// the number of choices beyond just two.
//

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
    let outer_edge      = merged_path.ray_collisions(&(outside_point, point_on_curve)).into_iter().nth(0).unwrap().0.into();

    // Trace the external edges from the path to find the exterior edges. We always follow an edge moving in the same direction on the other path.
    merged_path.classify_exterior_edges(outer_edge, |_graph, start_edge, edge_choices| {
        // Fetch the direction of the source edge
        let (source_path, source_direction) = start_edge.label();
        let source_reversed                 = start_edge.is_reversed();

        // At an intersection, the target must be on the other path
        let target_path = match source_path {
            SourcePath::Path1 => SourcePath::Path2,
            SourcePath::Path2 => SourcePath::Path1
        };

        for edge in edge_choices.into_iter() {
            let (edge_path, edge_direction) = edge.label();
            let edge_reversed               = edge.is_reversed();

            // Ignore edges from the source path
            if edge_path != target_path {
                continue;
            }

            if edge_reversed == source_reversed {
                // The target should be on the other path in the same direction
                if edge_direction == source_direction {
                    return edge.into();
                }
            } else {
                // Match against the opposite direction if the edge is reversed
                if edge_direction == source_direction.reversed() {
                    return edge.into();
                }
            }
        }

        panic!("Could not find a following edge")
    });

    // TODO: deal with holes, maybe also multiple paths meeting at a particular point?

    // Produce the final result
    merged_path.exterior_paths()
}
