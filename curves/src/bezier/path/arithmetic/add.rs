use super::arithmetic::*;
use super::super::path::*;
use super::super::graph_path::*;
use super::super::super::curve::*;
use super::super::super::normal::*;
use super::super::super::super::line::*;
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
        let outside_point = self.outside_point();

        loop {
            // Find a point on an uncategorised edge
            // We aim at the midpoint as if the ray hits an intersection, we can't easily tell which edge is exterior and which is interior (this means that we know the edge we're aiming at here won't be an intersection)
            let next_point = self.all_edges()
                .filter(|edge| edge.kind() == GraphPathEdgeKind::Uncategorised)
                .map(|edge| edge.point_at_pos(0.5))
                .nth(0);

            if let Some(next_point) = next_point {
                // Cast a ray to this point from the outside point and categorise any edges we encounter
                let collisions = self.ray_collisions(&(outside_point, next_point));

                // Collisions are ordered from the outer point, so we know the start of the line is outside the path
                let mut inside_path1 = false;
                let mut inside_path2 = false;

                for (collision, _curve_t, _line_t) in collisions {
                    // If the ray was in path1 or path2, it's coming from inside the combined shape
                    let was_inside      = inside_path1 || inside_path2;
                    let is_intersection = collision.is_intersection();

                    for edge in collision {
                        // Fetch information about these edges
                        let edge_kind                           = self.edge_kind(edge);
                        let PathLabel(source_path, _direction)  = self.edge_label(edge);

                        // Update the state of the ray. All source edges are considered to be exterior edges
                        match source_path {
                            PathSource::Path1 => { inside_path1 = !inside_path1 },
                            PathSource::Path2 => { inside_path2 = !inside_path2 }
                        }

                        // Intersections will have multiple edges which can need to be categorised differently
                        if !is_intersection {
                            // If the ray will be insde path1 or path2, then it's inside further on
                            let is_inside = inside_path1 || inside_path2;

                            // The edge is an exterior edge when crossing from inside to outside
                            let is_exterior = was_inside ^ is_inside;

                            // If the edge is uncategorised, categorise it
                            if edge_kind == GraphPathEdgeKind::Uncategorised {
                                if is_exterior {
                                    // Mark this edge and any connected to it as exterior
                                    self.set_edge_kind_connected(edge, GraphPathEdgeKind::Exterior);
                                } else {
                                    // Mark this edge and any connected to it as interior
                                    self.set_edge_kind_connected(edge, GraphPathEdgeKind::Interior);
                                }
                            }
                        }
                    }
                }
            } else {
                // All edges are categorised
                break;
            }
        }
    }

    ///
    /// Given a path that intersects itself (ie, only contains SourcePath::Path1), discovers the 'true' exterior edge.
    ///
    pub fn set_exterior_by_removing_interior_points(&mut self) {
        let outside_point = self.outside_point();

        loop {
            // Cast a ray at the next uncategorised edge
            let next_point = self.all_edges()
                .filter(|edge| edge.kind() == GraphPathEdgeKind::Uncategorised)
                .map(|edge| edge.point_at_pos(0.5))
                .nth(0);

            if let Some(next_point) = next_point {
                // The 'total direction' indicates how often we've crossed an edge moving in a particular direction
                // We're inside the path when it's non-zero
                let mut total_direction = 0;

                // Cast a ray at the target edge
                let ray         = (outside_point, next_point);
                let collisions  = self.ray_collisions(&ray);

                for (collision, curve_t, line_t) in collisions {
                    let is_intersection = collision.is_intersection();

                    for edge in collision {
                        let PathLabel(_path, direction) = self.edge_label(edge);

                        // The relative direction of the tangent to the ray indicates the direction we're crossing in
                        let pos     = ray.point_at_pos(line_t);
                        let tangent = self.get_edge(edge).tangent_at_pos(curve_t);

                        let side    = ray.which_side(&(pos+tangent));
                        let side    = match direction {
                            PathDirection::Clockwise        => { side },
                            PathDirection::Anticlockwise    => { -side }
                        };

                        let was_inside = total_direction != 0;
                        if side < 0 {
                            total_direction -= 1;
                        } else if side > 0 {
                            total_direction += 1;
                        }
                        let is_inside = total_direction != 0;

                        // If this isn't an intersection, set the edge's 
                        if !is_intersection {
                            // Exterior edges move from inside to outside or vice-versa
                            if was_inside ^ is_inside {
                                // Exterior edge
                                self.set_edge_kind_connected(edge, GraphPathEdgeKind::Exterior);
                            } else {
                                // Interior edge
                                self.set_edge_kind_connected(edge, GraphPathEdgeKind::Interior);
                            }
                        }
                    }
                }

            } else {
                // All edges are categorised
                break;
            }
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
    let mut merged_path = GraphPath::new();
    merged_path         = merged_path.merge(GraphPath::from_merged_paths(path1.into_iter().map(|path| (path, PathLabel(PathSource::Path1, PathDirection::from(path))))));

    // Collide with the target side to generate a full path
    merged_path         = merged_path.collide(GraphPath::from_merged_paths(path2.into_iter().map(|path| (path, PathLabel(PathSource::Path2, PathDirection::from(path))))), accuracy);

    // Set the exterior edges using the 'add' algorithm
    merged_path.set_exterior_by_adding();

    // Produce the final result
    merged_path.exterior_paths()
}

///
/// Generates the path formed by removing any interior points from an existing path
///
pub fn path_remove_interior_points<Point, P1: BezierPath<Point=Point>, POut: BezierPathFactory<Point=Point>>(path: &Vec<P1>) -> Vec<POut>
where   Point: Coordinate+Coordinate2D {
    // Create the graph path from the source side
    let mut merged_path = GraphPath::new();
    merged_path         = merged_path.merge(GraphPath::from_merged_paths(path.into_iter().map(|path| (path, PathLabel(PathSource::Path1, PathDirection::from(path))))));

    // Set the exterior edges using the 'add' algorithm
    merged_path.set_exterior_by_removing_interior_points();

    // Produce the final result
    merged_path.exterior_paths()
}
