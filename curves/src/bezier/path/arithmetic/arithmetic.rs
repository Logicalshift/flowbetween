use super::super::path::*;
use super::super::graph_path::*;
use super::super::is_clockwise::*;
use super::super::super::curve::*;
use super::super::super::normal::*;
use super::super::super::super::coordinate::*;

/// Source of a path in the graphpath
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PathSource {
    Path1,
    Path2
}

/// Target of a path in the graphpath
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PathDirection {
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

/// Label attached to a path used for arithmetic
#[derive(Clone, Copy, Debug)]
pub struct PathLabel(pub PathSource, pub PathDirection);

impl<Point: Coordinate+Coordinate2D> GraphPath<Point, PathLabel> {
    ///
    /// Sets the edge kinds by performing ray casting
    /// 
    /// The function passed in to this method takes two parameters: these are the number of times edges have been crossed in
    /// path 1 and path 2. It should return true if this number of crossings represents a point inside the final shape, or false
    /// if it represents a point outside of the shape.
    ///
    pub fn set_edge_kinds_by_ray_casting<FnIsInside: Fn(i32, i32) -> bool>(&mut self, is_inside: FnIsInside) {
        loop {
            // Cast a ray at the next uncategorised edge
            let next_point = self.all_edges()
                .filter(|edge| edge.kind() == GraphPathEdgeKind::Uncategorised)
                .map(|edge| (edge.point_at_pos(0.5), edge.normal_at_pos(0.5), edge.into()))
                .nth(0);

            if let Some((next_point, next_normal, next_edge)) = next_point {
                // Mark the next edge as visited (this prevents an infinite loop in the event the edge we're aiming at has a length of 0 and thus will always be an intersection)
                self.set_edge_kind(next_edge, GraphPathEdgeKind::Visited);

                // The 'total direction' indicates how often we've crossed an edge moving in a particular direction
                // We're inside the path when it's non-zero
                let mut path1_crossings = 0;
                let mut path2_crossings = 0;

                // Cast a ray at the target edge
                let ray             = (next_point - next_normal, next_point);
                let ray_direction   = ray.1 - ray.0;
                let collisions      = self.ray_collisions(&ray);

                // There should always be an even number of collisions on a particular ray cast through a closed shape
                debug_assert!((collisions.len()&1) == 0);

                for (collision, curve_t, _line_t, _pos) in collisions {
                    let is_intersection = collision.is_intersection();
                    let edge            = collision.edge();

                    let PathLabel(path, direction) = self.edge_label(edge);

                    // The relative direction of the tangent to the ray indicates the direction we're crossing in
                    let normal  = self.get_edge(edge).normal_at_pos(curve_t);

                    let side    = ray_direction.dot(&normal).signum() as i32;
                    let side    = match direction {
                        PathDirection::Clockwise        => { side },
                        PathDirection::Anticlockwise    => { -side }
                    };

                    let was_inside = is_inside(path1_crossings, path2_crossings);
                    if side < 0 {
                        match path {
                            PathSource::Path1 => { path1_crossings -= 1 },
                            PathSource::Path2 => { path2_crossings -= 1 }
                        }
                    } else if side > 0 {
                        match path {
                            PathSource::Path1 => { path1_crossings += 1 },
                            PathSource::Path2 => { path2_crossings += 1 }
                        }
                    }
                    let is_inside = is_inside(path1_crossings, path2_crossings);

                    // If this isn't an intersection, set whether or not the edge is exterior
                    let edge_kind = self.edge_kind(edge);
                    if !is_intersection && (edge_kind == GraphPathEdgeKind::Uncategorised || edge_kind == GraphPathEdgeKind::Visited) {
                        // Exterior edges move from inside to outside or vice-versa
                        if was_inside ^ is_inside {
                            // Exterior edge
                            self.set_edge_kind(edge, GraphPathEdgeKind::Exterior);
                        } else {
                            // Interior edge
                            self.set_edge_kind(edge, GraphPathEdgeKind::Interior);
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
