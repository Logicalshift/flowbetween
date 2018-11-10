use super::graph_path::*;
use super::super::curve::*;
use super::super::intersection::*;
use super::super::super::line::*;
use super::super::super::consts::*;
use super::super::super::coordinate::*;

///
/// Returns true if a curve is collinear given the set of coefficients for a ray
///
#[inline]
fn curve_is_collinear<Edge: BezierCurve>(edge: &Edge, (a, b, c): (f64, f64, f64)) -> bool {
    // Fetch the points of the curve
    let start_point = edge.start_point();
    let end_point   = edge.end_point();
    let (cp1, cp2)  = edge.control_points();

    // The curve is collinear if all of the points lie on the 
    if (start_point.x()*a + start_point.y()*b + c).abs() < SMALL_DISTANCE
    && (end_point.x()*a + end_point.y()*b + c).abs() < SMALL_DISTANCE
    && (cp1.x()*a + cp1.y()*b + c).abs() < SMALL_DISTANCE
    && (cp2.x()*a + cp2.y()*b + c).abs() < SMALL_DISTANCE {
        true
    } else {
        false
    }
}

///
/// Given the coefficients of a ray, returns whether or not an edge can intersect it
///
#[inline]
fn ray_can_intersect<Edge: BezierCurve>(edge: &Edge, (a, b, c): (f64, f64, f64)) -> bool {
    // Fetch the points of the curve
    let start_point = edge.start_point();
    let end_point   = edge.end_point();
    let (cp1, cp2)  = edge.control_points();
    
    let side        = (a*start_point.x() + b*start_point.y() + c).signum()
                    + (a*cp1.x() + b*cp1.y() + c).signum()
                    + (a*cp2.x() + b*cp2.y() + c).signum()
                    + (a*end_point.x()+ b*end_point.y() + c).signum();

    // If all 4 points have the same sign, they're all on the same side of the ray and thus the edge cannot intersect it 
    if side < -3.99 || side > 3.99 {
        false
    } else {
        true
    }
}

///
/// Given a list of points, returns the edges that cross the line given by the specified set of coefficients
///
fn crossing_edges<'a>(&'a self, (a, b, c): (f64, f64, f64), points: Vec<usize>) -> Vec<GraphEdge<'a, Point, Label>> {
    let mut crossing_edges = vec![];

    for point_idx in points.into_iter() {
        for incoming in self.reverse_edges_for_point(point_idx) {
            // Get the incoming edge going in the right direction
            let incoming    = incoming.reversed();

            // Ignore collinear incoming edges
            if curve_is_collinear(&incoming, (a, b, c)) {
                continue;
            }

            // Fetch the leaving edge for the incoming edge
            let leaving     = GraphEdgeRef { start_idx: point_idx, edge_idx: incoming.following_edge_idx(), reverse: false };
            let mut leaving = GraphEdge::new(self, leaving);

            // Follow the path until we complete a loop or find a leaving edge that's not collinear
            while curve_is_collinear(&leaving, (a, b, c)) {
                leaving = leaving.next_edge();

                if leaving.start_point_index() == point_idx {
                    // Found a loop that was entirely collinear
                    // (Provided that the following edges always form a closed path this should always be reached, which is currently always true for the means we have to create a graph path)
                    break;
                }
            }

            // If it's not colinear, add to the set of crossing edges
            if !curve_is_collinear(&leaving, (a, b, c)) {
                let incoming_cp2    = incoming.control_points().1;
                let leaving_cp1     = leaving.control_points().0;

                let incoming_side   = a*incoming_cp2.x() + b*incoming_cp2.y() + c;
                let leaving_side    = a*leaving_cp1.x() + b*leaving_cp1.y() + c;

                if incoming_side.signum() != leaving_side.signum() {
                    // Control points are on different sides of the line, so this is a crossing edge
                    crossing_edges.push(leaving);
                }
            }
        }
    }

    crossing_edges
}

///
/// Takes a ray and collides it against every edge in this path, returning a list of collisions
///
#[inline]
fn raw_ray_collisions<'a, L: Line<Point=Point>>(&'a self, ray: &'a L) -> impl 'a+Iterator<Item=(GraphEdgeRef, f64, f64, Point)> {
    let ray_coeffs  = ray.coefficients();

    self.all_edges()
        .filter(move |edge| !curve_is_collinear(&edge, ray_coeffs))
        .filter(move |edge| ray_can_intersect(&edge, ray_coeffs))
        .flat_map(move |edge| curve_intersects_ray(&edge, ray)
                .into_iter()
                .map(move |(curve_t, line_t, collide_pos)| (GraphEdgeRef::from(&edge), curve_t, line_t, collide_pos)))
}

///
/// Takes a ray and collides it against every collinear edge in this path, returning the list of edges that cross the collinear
/// section (collinear edges have 0 width so can't be crossed themselves)
///
#[inline]
fn collinear_ray_collisions<'a, L: Line<Point=Point>>(&'a self, ray: &'a L) -> impl 'a+Iterator<Item=(GraphEdgeRef, f64, f64, Point)> {
    let ray_coeffs = ray.coefficients();

    // Find all of the collinear sections (sets of points connected by collinear edges)
    let mut section_with_point: Vec<Option<usize>>  = vec![None; self.points.len()];
    let mut collinear_sections: Vec<Vec<_>>         = vec![];

    for edge in self.all_edges().filter(|edge| curve_is_collinear(&edge, ray_coeffs)) {
        let start_idx   = edge.start_point_index();
        let end_idx     = edge.end_point_index();

        if let Some(start_section) = section_with_point[start_idx] {
            if let Some(_end_section) = section_with_point[end_idx] {
                // Already seen an edge between these points
            } else {
                // end_idx is new
                collinear_sections[start_section].push(end_idx);
            }
        } else if let Some(end_section) = section_with_point[end_idx] {
            // start_idx is new
            collinear_sections[end_section].push(start_idx);
        } else {
            // New section
            let new_section = collinear_sections.len();
            collinear_sections.push(vec![start_idx, end_idx]);
            section_with_point[start_idx]   = Some(new_section);
            section_with_point[end_idx]     = Some(new_section);
        }
    }

    // Find the edges crossing each collinear section
    collinear_sections
        .into_iter()
        .flat_map(move |colinear_edge_points| self.crossing_edges(ray_coeffs, colinear_edge_points)
                .into_iter()
                .map(move |crossing_edge| {
                    let point   = crossing_edge.start_point();
                    let line_t  = ray.pos_for_point(&point);

                    (GraphEdgeRef::from(&crossing_edge), 0.0, line_t, point)
                }))
}

///
/// Given a list of collisions, removes any that are at the end just before a collinear section
///
#[inline]
fn remove_collisions_before_or_after_collinear_section<'a, L: Line<Point=Point>, Collisions: 'a+IntoIterator<Item=(GraphEdgeRef, f64, f64, Point)>>(&'a self, ray: &L, collisions: Collisions) -> impl 'a+Iterator<Item=(GraphEdgeRef, f64, f64, Point)> {
    let ray_coeffs = ray.coefficients();

    collisions.into_iter()
        .filter(move |(collision, curve_t, _line_t, position)| {
            if *curve_t > 0.9 {
                let edge = GraphEdge::new(self, *collision);

                // If any following edge is collinear, remove this collision
                if position.is_near_to(&edge.end_point(), SMALL_DISTANCE) && self.edges_for_point(edge.end_point_index()).any(|next| curve_is_collinear(&next, ray_coeffs)) {
                    false
                } else {
                    true
                }
            } else if *curve_t < 0.1 {
                let edge = GraphEdge::new(self, *collision);

                // If any preceding edge is collinear, remove this collision
                if position.is_near_to(&edge.start_point(), SMALL_DISTANCE) && self.reverse_edges_for_point(collision.start_idx).any(|previous| curve_is_collinear(&previous, ray_coeffs)) {
                    // Collisions crossing collinear sections are taken care of during the collinear collision phase
                    false
                } else {
                    true
                }
            } else {
                // Not at the end of a curve
                true
            }
        })
}

///
/// Given a list of collisions, finds the collisions that occurred at the end of an edge and move them to the beginning of the next edge
///
#[inline]
fn move_collisions_at_end_to_beginning<'a, Collisions: 'a+IntoIterator<Item=(GraphEdgeRef, f64, f64, Point)>>(&'a self, collisions: Collisions) -> impl 'a+Iterator<Item=(GraphEdgeRef, f64, f64, Point)> {
    collisions.into_iter()
        .map(move |(collision, curve_t, line_t, position)| {
            if curve_t > 0.99999 {
                // Collisions at the very end of the curve should be considered to be at the start of the following curve
                // (as a ray intersecting a point will collide with both the previous and next curve)
                let next_point_idx  = self.points[collision.start_idx].forward_edges[collision.edge_idx].end_idx;

                if self.points[next_point_idx].position.is_near_to(&position, SMALL_DISTANCE) {
                    // Very close to the end of the curve
                    let collision = GraphEdgeRef {
                        start_idx:  self.points[collision.start_idx].forward_edges[collision.edge_idx].end_idx,
                        edge_idx:   self.points[collision.start_idx].forward_edges[collision.edge_idx].following_edge_idx,
                        reverse:    false,
                    };
                    (collision, 0.0, line_t, position)
                } else {
                    // Not at the end of a curve
                    (collision, curve_t, line_t, position)
                }
            } else if curve_t < 0.00001 {
                // Also check for points very close to the start of a curve and move those
                if self.points[collision.start_idx].position.is_near_to(&position, SMALL_DISTANCE) {
                    // Very close to the start of the curve
                    (collision, 0.0, line_t, position)
                } else {
                    // Not at the start of a curve
                    (collision, curve_t, line_t, position)
                }
            } else {
                // Not at the end of a curve
                (collision, curve_t, line_t, position)
            }
        })
}

///
/// Given a list of collisions, finds any that are on a collinear line and moves them to the end of the collinear section
///
#[inline]
fn move_collinear_collisions_to_end<'a, L: Line<Point=Point>, Collisions: 'a+IntoIterator<Item=(GraphEdgeRef, f64, f64, Point)>>(&'a self, ray: &L, collisions: Collisions) -> impl 'a+Iterator<Item=(GraphEdgeRef, f64, f64, Point)> {
    let ray_coeffs = ray.coefficients();

    collisions.into_iter()
        .map(move |(collision, curve_t, line_t, position)| {
            let edge = GraphEdge::new(self, collision);
            if curve_is_collinear(&edge, ray_coeffs) {
                let mut edge = edge;

                loop {
                    edge = edge.next_edge();
                    if !curve_is_collinear(&edge, ray_coeffs) {
                        break;
                    }
                }

                let position = edge.start_point();
                (edge.into(), 0.0, line_t, position)
            } else {
                (collision, curve_t, line_t, position)
            }
        })
}

///
/// Removes collisions that do not appear to enter the shape
///
#[inline]
fn remove_glancing_collisions<'a, L: Line<Point=Point>, Collisions: 'a+IntoIterator<Item=(GraphEdgeRef, f64, f64, Point)>>(&'a self, ray: &L, collisions: Collisions) -> impl 'a+Iterator<Item=(GraphEdgeRef, f64, f64, Point)> {
    let (a, b, c) = ray.coefficients();

    collisions
        .into_iter()
        .filter(move |(collision, curve_t, _line_t, _position)| {
            if *curve_t <= 0.000 {
                // Find the edge before this one
                let edge            = GraphEdge::new(self, *collision);
                let previous_edge   = self.reverse_edges_for_point(collision.start_idx)
                    .map(|edge| edge.reversed())
                    .filter(|edge| edge.following_edge_idx() == collision.edge_idx)
                    .nth(0)
                    .expect("Previous edge for glancing collision");

                // A glancing collision has control points on the same side of the ray
                let cp_in   = previous_edge.control_points().1;
                let cp_out  = edge.control_points().0;

                let side_in     = cp_in.x()*a + cp_in.y()*b + c;
                let side_out    = cp_out.x()*a + cp_out.y()*b + c;

                let side_in     = if side_in.abs() < 0.001 { 0.0 } else { side_in.signum() };
                let side_out    = if side_out.abs() < 0.001 { 0.0 } else { side_out.signum() };

                side_in != side_out
            } else {
                true
            }
        })
}

///
/// Finds any collision in the source that's at the start of its curve and filters so that only a single version is returned
/// 
/// (A collision exactly at the start of an edge will produce two collisions: one of the incoming edge and one on the outgoing one)
///
#[inline]
fn remove_duplicate_collisions_at_start<'a, Collisions: 'a+IntoIterator<Item=(GraphEdgeRef, f64, f64, Point)>>(&'a self, collisions: Collisions) -> impl 'a+Iterator<Item=(GraphEdgeRef, f64, f64, Point)> {
    let mut visited_start = vec![vec![]; self.points.len()];

    collisions
        .into_iter()
        .filter(move |(collision, curve_t, _line_t, _position)| {
            if *curve_t <= 0.000 {
                // At the start of the curve
                let was_visited = visited_start[collision.start_idx].contains(&collision.edge_idx);

                if !was_visited {
                    visited_start[collision.start_idx].push(collision.edge_idx);
                }

                !was_visited
            } else {
                // Not at the start of the curve
                true
            }
        })
}

///
/// Finds any collision that occurred too close to an intersection and flags it as such
///
#[inline]
fn flag_collisions_at_intersections<'a, Collisions: 'a+IntoIterator<Item=(GraphEdgeRef, f64, f64, Point)>>(&'a self, collisions: Collisions) -> impl 'a+Iterator<Item=(GraphRayCollision, f64, f64, Point)> {
    collisions
        .into_iter()
        .map(move |(collision, curve_t, line_t, position)| {
            if curve_t <= 0.000 {
                // Might be at an intersection (close to the start of the curve)
                if self.points[collision.start_idx].forward_edges.len() > 1 {
                    // Intersection
                    (GraphRayCollision::Intersection(collision), curve_t, line_t, position)
                } else {
                    // Edge with only a single following point
                    (GraphRayCollision::SingleEdge(collision), curve_t, line_t, position)
                }
            } else {
                // Not at an intersection
                (GraphRayCollision::SingleEdge(collision), curve_t, line_t, position)
            }
        })
}

///
/// Finds all collisions between a ray and this path
/// 
pub fn ray_collisions<L: Line<Point=Point>>(&self, ray: &L) -> Vec<(GraphRayCollision, f64, f64, Point)> {
    // Raw collisions
    let collinear_collisions    = self.collinear_ray_collisions(ray);
    let crossing_collisions     = self.raw_ray_collisions(ray);
    let crossing_collisions     = self.remove_collisions_before_or_after_collinear_section(ray, crossing_collisions);

    // Chain them together
    let collisions = collinear_collisions.chain(crossing_collisions);

    // Filter for accuracy
    let collisions = self.move_collisions_at_end_to_beginning(collisions);
    let collisions = self.move_collinear_collisions_to_end(ray, collisions);
    let collisions = self.remove_glancing_collisions(ray, collisions);
    let collisions = self.remove_duplicate_collisions_at_start(collisions);
    let collisions = self.flag_collisions_at_intersections(collisions);

    // Convert to a vec and sort by ray position
    let mut collisions = collisions.collect::<Vec<_>>();

    collisions.sort_by(|(edge_a, _curve_t_a, line_t_a, _pos_a), (edge_b, _curve_t_b, line_t_b, _pos_b)| {
        let result = line_t_a.partial_cmp(line_t_b).unwrap_or(Ordering::Equal);

        if result != Ordering::Equal {
            // Position on the line is different
            result
        } else {
            // Position on the line is the same (stabilise ordering by checking the edges)
            let edge_a = edge_a.edge();
            let edge_b = edge_b.edge();

            let result = edge_a.start_idx.cmp(&edge_b.start_idx);
            if result != Ordering::Equal {
                // Different start points
                result
            } else {
                // Check if these are the same edge or not
                edge_a.edge_idx.cmp(&edge_b.edge_idx)
            }
        }
    });

    collisions
}

