use super::path::*;
use super::super::curve::*;
use super::super::intersection::*;
use super::super::super::geo::*;
use super::super::super::line::*;
use super::super::super::consts::*;
use super::super::super::coordinate::*;

use std::fmt;
use std::mem;
use std::ops::Range;
use std::cmp::Ordering;

///
/// Represents a collision between a ray and a GraphPath
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GraphRayCollision {
    /// Collision against a single edge
    SingleEdge(GraphEdgeRef),

    /// Collision against an intersection point
    Intersection(GraphEdgeRef)
}

///
/// Kind of a graph path edge
/// 
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GraphPathEdgeKind {
    /// An edge that hasn't been categorised yet
    Uncategorised,

    /// An edge that is uncategorised but has been visited
    Visited,

    /// An exterior edge
    /// 
    /// These edges represent a transition between the inside and the outside of the path
    Exterior, 

    /// An interior edge
    /// 
    /// These edges are on the inside of the path
    Interior
}

///
/// Reference to a graph edge
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphEdgeRef {
    /// The index of the point this edge starts from
    start_idx: usize,

    /// The index of the edge within the point
    edge_idx: usize,

    /// True if this reference is for the reverse of this edge
    reverse: bool
}

///
/// Enum representing an edge in a graph path
/// 
#[derive(Clone, Debug)]
struct GraphPathEdge<Point, Label> {
    /// The label attached to this edge
    label: Label,

    /// The ID of the edge following this one on the target point
    following_edge_idx: usize,

    /// The kind of this edge
    kind: GraphPathEdgeKind,

    /// Position of the first control point
    cp1: Point,

    /// Position of the second control point
    cp2: Point,

    /// The index of the target point
    end_idx: usize
}

///
/// Struct representing a point in a graph path
///
#[derive(Clone, Debug)]
struct GraphPathPoint<Point, Label> {
    /// The position of this point
    position: Point,

    /// The edges attached to this point
    forward_edges: Vec<GraphPathEdge<Point, Label>>,

    /// The points with edges connecting to this point
    connected_from: Vec<usize>
}

///
/// Struct representing a collision in the graph path
///
struct Collision {
    idx:    usize,
    edge:   usize,
    t:      f64
}

///
/// Struct representing a set of collisions in the graph path
///
struct CollisionList {
    /// List of collisions on the source and target side
    collisions: Vec<(Collision, Collision)>
}

impl CollisionList {
    ///
    /// Creates a new list of collisions
    ///
    fn new() -> CollisionList {
        CollisionList { 
            collisions: vec![]
        }
    }

    ///
    /// Adds a collision to this list
    ///
    fn push(&mut self, collision: (Collision, Collision)) {
        self.collisions.push(collision);
    }

    ///
    /// Removes the last collision from this list
    ///
    fn pop(&mut self) -> Option<(Collision, Collision)> {
        self.collisions.pop()
    }

    ///
    /// For all remaining collisions, finds any that use the specified edge and change them so they are subdivided at 
    /// the specified t value
    ///
    fn move_after_midpoint<Point, Label>(&mut self, graph: &mut GraphPath<Point, Label>, midpoint: usize, point_idx: usize, edge_idx: usize, t: f64, new_edge_idx: usize) {
        // Usually new_mid_point is a new point, but it can be an existing point in the event the collision was at an existing point on the path
        debug_assert!(midpoint < graph.points.len());
        debug_assert!(new_edge_idx < graph.points[midpoint].forward_edges.len());

        // TODO(?): this just iterates through the collisions, not clear if this will always be fast enough
        for (ref mut collision_src, ref mut collision_tgt) in self.collisions.iter_mut() {
            // If the src edge was divided...
            if collision_src.idx == point_idx && collision_src.edge == edge_idx {
                if collision_src.t < t {
                    // Before the midpoint. Edge is the same, just needs to be modified.
                    collision_src.t /= t;
                } else {
                    debug_assert!(graph.points[midpoint].forward_edges.len() > 0);

                    // After the midpoint. Edge needs to be adjusted.
                    collision_src.t     = (collision_src.t - t) / (1.0-t);
                    collision_src.idx   = midpoint;
                    collision_src.edge  = new_edge_idx;
                }
            }

            // If the target edge was divided...
            if collision_tgt.idx == point_idx && collision_tgt.edge == edge_idx {
                if collision_tgt.t < t {
                    // Before the midpoint. Edge is the same, just needs to be modified.
                    collision_tgt.t /= t;
                } else {
                    debug_assert!(graph.points[midpoint].forward_edges.len() > 1);

                    // After the midpoint. Edge needs to be adjusted.
                    collision_tgt.t     = (collision_tgt.t - t) / (1.0-t);
                    collision_tgt.idx   = midpoint;
                    collision_tgt.edge  = new_edge_idx;
                }
            }
        }
    }

    ///
    /// Takes all the collisions that were originally on `original_point_idx` and changes them to `new_point_idx`.
    /// The edges should still be in sequence, starting at `edge_idx_offset` in the new point
    ///
    fn move_all_edges(&mut self, original_point_idx: usize, new_point_idx: usize, edge_idx_offset: usize) {
        if original_point_idx == new_point_idx {
            // Edges will be unchanged
            return;
        }

        for (ref mut collision_src, ref mut collision_tgt) in self.collisions.iter_mut() {
            if collision_src.idx == original_point_idx {
                collision_src.idx   = new_point_idx;
                collision_src.edge  += edge_idx_offset;
            }
            if collision_tgt.idx == original_point_idx {
                collision_tgt.idx   = new_point_idx;
                collision_tgt.edge  += edge_idx_offset;
            }
        }
    }

    ///
    /// Checks consistency of the points and edges against a graph path
    ///
    #[cfg(debug_assertions)]
    fn check_consistency<Point, Label>(&self, graph: &GraphPath<Point, Label>) {
        for (src, tgt) in self.collisions.iter() {
            debug_assert!(src.idx < graph.points.len());
            debug_assert!(src.edge < graph.points[src.idx].forward_edges.len());

            debug_assert!(tgt.idx < graph.points.len());
            debug_assert!(tgt.edge < graph.points[tgt.idx].forward_edges.len());
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    fn check_consistency<Point, Label>(&self, _graph: &GraphPath<Point, Label>) {
    }
}

impl<Point, Label> GraphPathPoint<Point, Label> {
    ///
    /// Creates a new graph path point
    ///
    fn new(position: Point, forward_edges: Vec<GraphPathEdge<Point, Label>>, connected_from: Vec<usize>) -> GraphPathPoint<Point, Label> {
        GraphPathPoint { position, forward_edges, connected_from }
    }
}

impl<Point: Coordinate, Label> GraphPathEdge<Point, Label> {
    ///
    /// Creates a new graph path edge
    /// 
    #[inline]
    fn new(kind: GraphPathEdgeKind, (cp1, cp2): (Point, Point), end_idx: usize, label: Label, following_edge_idx: usize) -> GraphPathEdge<Point, Label> {
        GraphPathEdge {
            label, kind, cp1, cp2, end_idx, following_edge_idx
        }
    }

    ///
    /// Updates the control points of this edge
    /// 
    #[inline]
    fn set_control_points(&mut self, (cp1, cp2): (Point, Point), end_idx: usize, next_edge_idx: usize) {
        self.cp1                = cp1;
        self.cp2                = cp2;
        self.end_idx            = end_idx;
        self.following_edge_idx = next_edge_idx;
    }
}

///
/// A graph path is a path where each point can have more than one connected edge. Edges are categorized
/// into interior and exterior edges depending on if they are on the outside or the inside of the combined
/// shape.
/// 
#[derive(Clone)]
pub struct GraphPath<Point, Label> {
    /// The points in this graph and their edges. Each 'point' here consists of two control points and an end point
    points: Vec<GraphPathPoint<Point, Label>>,

    /// The index to assign to the next path added to this path
    next_path_index: usize
}

impl<Point: Coordinate, Label> Geo for GraphPath<Point, Label> {
    type Point = Point;
}

impl<Point: Coordinate+Coordinate2D, Label: Copy> GraphPath<Point, Label> {
    ///
    /// Creates a new graph path with no points
    ///
    pub fn new() -> GraphPath<Point, Label> {
        GraphPath {
            points:             vec![],
            next_path_index:    0
        }
    }

    ///
    /// Creates a graph path from a bezier path
    /// 
    pub fn from_path<P: BezierPath<Point=Point>>(path: &P, label: Label) -> GraphPath<Point, Label> {
        // All edges are exterior for a single path
        let mut points = vec![];

        // Push the start point (with an open path)
        let start_point = path.start_point();
        points.push(GraphPathPoint::new(start_point, vec![], vec![]));

        // We'll add edges to the previous point
        let mut last_point_pos  = start_point;
        let mut last_point_idx  = 0;
        let mut next_point_idx  = 1;

        // Iterate through the points in the path
        for (cp1, cp2, end_point) in path.points() {
            // Ignore points that are too close to the last point
            if end_point.is_near_to(&last_point_pos, CLOSE_DISTANCE) {
                if cp1.is_near_to(&last_point_pos, CLOSE_DISTANCE) && cp2.is_near_to(&cp1, CLOSE_DISTANCE) {
                    continue;
                }
            }

            // Push the points
            points.push(GraphPathPoint::new(end_point, vec![], vec![]));

            // Add an edge from the last point to the next point
            points[last_point_idx].forward_edges.push(GraphPathEdge::new(GraphPathEdgeKind::Uncategorised, (cp1, cp2), next_point_idx, label, 0));

            // Update the last/next pooints
            last_point_idx  += 1;
            next_point_idx  += 1;
            last_point_pos  = end_point;
        }

        // Close the path
        if last_point_idx > 0 {
            // Graph actually has some edges
            if start_point.distance_to(&points[last_point_idx].position) < CLOSE_DISTANCE {
                // Remove the last point (we're replacing it with an edge back to the start)
                points.pop();
                last_point_idx -= 1;

                // Change the edge to point back to the start
                points[last_point_idx].forward_edges[0].end_idx = 0;
            } else {
                // Need to draw a line to the last point (as there is always a single following edge, the following edge index is always 0 here)
                let close_vector    = points[last_point_idx].position - start_point;
                let cp1             = close_vector * 0.33 + start_point;
                let cp2             = close_vector * 0.66 + start_point;

                points[last_point_idx].forward_edges.push(GraphPathEdge::new(GraphPathEdgeKind::Uncategorised, (cp1, cp2), 0, label, 0));
            }
        } else {
            // Just a start point and no edges: remove the start point as it doesn't really make sense
            points.pop();
        }

        // Create the graph path from the points
        let mut path = GraphPath {
            points:             points,
            next_path_index:    1
        };
        path.recalculate_reverse_connections();
        path
    }

    ///
    /// Creates a new graph path by merging (not colliding) a set of paths with their labels
    ///
    pub fn from_merged_paths<'a, P: 'a+BezierPath<Point=Point>, PathIter: IntoIterator<Item=(&'a P, Label)>>(paths: PathIter) -> GraphPath<Point, Label> {
        // Create an empty path
        let mut merged_path = GraphPath::new();

        // Merge each path in turn
        for (path, label) in paths {
            let path    = GraphPath::from_path(path, label);
            merged_path = merged_path.merge(path);
        }

        merged_path
    }

    ///
    /// Recomputes the list of items that have connections to each point
    ///
    fn recalculate_reverse_connections(&mut self) {
        // Reset the list of connections to be empty
        for point_idx in 0..(self.points.len()) {
            self.points[point_idx].connected_from = vec![];
        }

        // Add a reverse connection for every edge
        for point_idx in 0..(self.points.len()) {
            for edge_idx in 0..(self.points[point_idx].forward_edges.len()) {
                let end_idx = self.points[point_idx].forward_edges[edge_idx].end_idx;
                self.points[end_idx].connected_from.push(point_idx);
            }
        }
    }

    ///
    /// Returns the number of points in this graph. Points are numbered from 0 to this value.
    /// 
    #[inline]
    pub fn num_points(&self) -> usize {
        self.points.len()
    }

    ///
    /// Returns an iterator of all edges in this graph
    ///
    #[inline]
    pub fn all_edges<'a>(&'a self) -> impl 'a+Iterator<Item=GraphEdge<'a, Point, Label>> {
        (0..(self.points.len()))
            .into_iter()
            .flat_map(move |point_num| self.edges_for_point(point_num))
    }

    ///
    /// Returns an iterator of all the edges in this graph, as references
    ///
    #[inline]
    pub fn all_edge_refs<'a>(&'a self) -> impl 'a+Iterator<Item=GraphEdgeRef> {
        (0..(self.points.len()))
            .into_iter()
            .flat_map(move |point_idx| (0..(self.points[point_idx].forward_edges.len()))
                .into_iter()
                .map(move |edge_idx| GraphEdgeRef {
                    start_idx:  point_idx,
                    edge_idx:   edge_idx,
                    reverse:    false
                }))
    }

    ///
    /// Returns an iterator of the edges that leave a particular point
    /// 
    /// Edges are directional: this will provide the edges that leave the supplied point
    ///
    #[inline]
    pub fn edges_for_point<'a>(&'a self, point_num: usize) -> impl 'a+Iterator<Item=GraphEdge<'a, Point, Label>> {
        (0..(self.points[point_num].forward_edges.len()))
            .into_iter()
            .map(move |edge_idx| GraphEdge::new(self, GraphEdgeRef { start_idx: point_num, edge_idx: edge_idx, reverse: false }))
    }

    ///
    /// Returns the position of a particular point
    ///
    #[inline]
    pub fn point_position(&self, point_num: usize) -> Point {
        self.points[point_num].position.clone()
    }

    ///
    /// Returns an iterator of the edges that arrive at a particular point
    /// 
    /// Edges are directional: this will provide the edges that connect to the supplied point
    ///
    pub fn reverse_edges_for_point<'a>(&'a self, point_num: usize) -> impl 'a+Iterator<Item=GraphEdge<'a, Point, Label>> {
        // Fetch the points that connect to this point
        self.points[point_num].connected_from
            .iter()
            .flat_map(move |connected_from| {
                let connected_from = *connected_from;

                // Any edge that connects to the current point, in reverse
                (0..(self.points[connected_from].forward_edges.len()))
                    .into_iter()
                    .filter_map(move |edge_idx| {
                        if self.points[connected_from].forward_edges[edge_idx].end_idx == point_num {
                            Some(GraphEdgeRef { start_idx: connected_from, edge_idx: edge_idx, reverse: true })
                        } else {
                            None
                        }
                    })
            })
            .map(move |edge_ref| GraphEdge::new(self, edge_ref))
    }

    ///
    /// Merges in another path
    /// 
    /// This adds the edges in the new path to this path without considering if they are internal or external 
    ///
    pub fn merge(self, merge_path: GraphPath<Point, Label>) -> GraphPath<Point, Label> {
        // Copy the points from this graph
        let mut new_points  = self.points;
        let next_path_idx   = self.next_path_index;

        // Add in points from the merge path
        let offset          = new_points.len();
        new_points.extend(merge_path.points.into_iter()
            .map(|mut point| {
                // Update the offsets in the edges
                for mut edge in &mut point.forward_edges {
                    edge.end_idx            += offset;
                }

                for mut previous_point in &mut point.connected_from {
                    *previous_point += offset;
                }

                // Generate the new edge
                point
            }));

        // Combined path
        GraphPath {
            points:             new_points,
            next_path_index:    next_path_idx + merge_path.next_path_index
        }
    }

    /// 
    /// True if the t value is effectively at the start of the curve
    /// 
    #[inline]
    fn t_is_zero(t: f64) -> bool { t < SMALL_T_DISTANCE }

    ///
    /// True if the t value is effective at the end of the curve
    /// 
    #[inline]
    fn t_is_one(t: f64) -> bool { t > (1.0-SMALL_T_DISTANCE) }

    ///
    /// Joins two edges at an intersection, returning the index of the intersection point
    /// 
    /// For t=0 or 1 the intersection point may be one of the ends of the edges, otherwise
    /// this will divide the existing edges so that they both meet at the specified mid-point.
    /// 
    /// Note that the case where t=1 is the same as the case where t=0 on a following edge.
    /// The split algorithm is simpler if only the t=0 case is considered.
    /// 
    #[inline]
    fn join_edges_at_intersection(&mut self, edge1: (usize, usize), edge2: (usize, usize), t1: f64, t2: f64, collisions: &mut CollisionList) -> Option<usize> {
        // Do nothing if the edges are the same (they're effectively already joined)
        if edge1 == edge2 { return None; }

        // Get the edge indexes
        let (edge1_idx, edge1_edge_idx) = edge1;
        let (edge2_idx, edge2_edge_idx) = edge2;

        // Create representations of the two edges
        let edge1 = Curve::from_curve(&GraphEdge::new(self, GraphEdgeRef { start_idx: edge1_idx, edge_idx: edge1_edge_idx, reverse: false }));
        let edge2 = Curve::from_curve(&GraphEdge::new(self, GraphEdgeRef { start_idx: edge2_idx, edge_idx: edge2_edge_idx, reverse: false }));

        // If we're very close to the start or end, round to the start/end
        let p1  = edge1.point_at_pos(t1);
        let p2  = edge2.point_at_pos(t2);

        let t1  = if p1.is_near_to(&edge1.start_point(), SMALL_DISTANCE) {
            0.0
        } else if p1.is_near_to(&edge1.end_point(), SMALL_DISTANCE) {
            1.0
        } else {
            t1
        };

        let t2  = if p2.is_near_to(&edge2.start_point(), SMALL_DISTANCE) {
            0.0
        } else if p2.is_near_to(&edge2.end_point(), SMALL_DISTANCE) {
            1.0
        } else {
            t2
        };

        // Create or choose a point to collide at
        // (If t1 or t2 is 0 or 1 we collide on the edge1 or edge2 points, otherwise we create a new point to collide at)
        let collision_point = if Self::t_is_zero(t1) {
            edge1_idx
        } else if Self::t_is_one(t1) {
            self.points[edge1_idx].forward_edges[edge1_edge_idx].end_idx
        } else if Self::t_is_zero(t2) {
            edge2_idx
        } else if Self::t_is_one(t2) {
            self.points[edge2_idx].forward_edges[edge2_edge_idx].end_idx
        } else {
            // Point is a mid-point of both lines

            // Work out where the mid-point is (use edge1 for this always: as this is supposed to be an intersection this shouldn't matter)
            // Note that if we use de Casteljau's algorithm here we get a subdivision for 'free' but organizing the code around it is painful
            let mid_point = edge1.point_at_pos(t1);

            // Add to this list of points
            let mid_point_idx = self.points.len();
            self.points.push(GraphPathPoint::new(mid_point, vec![], vec![]));

            // New point is the mid-point
            mid_point_idx
        };

        // Subdivide the edges
        let (edge1a, edge1b) = edge1.subdivide::<Curve<_>>(t1);
        let (edge2a, edge2b) = edge2.subdivide::<Curve<_>>(t2);

        // The new edges have the same kinds as their ancestors
        let edge1_kind          = self.points[edge1_idx].forward_edges[edge1_edge_idx].kind;
        let edge2_kind          = self.points[edge2_idx].forward_edges[edge2_edge_idx].kind;
        let edge1_label         = self.points[edge1_idx].forward_edges[edge1_edge_idx].label;
        let edge2_label         = self.points[edge2_idx].forward_edges[edge2_edge_idx].label;
        let edge1_end_idx       = self.points[edge1_idx].forward_edges[edge1_edge_idx].end_idx;
        let edge2_end_idx       = self.points[edge2_idx].forward_edges[edge2_edge_idx].end_idx;
        let edge1_following_idx = self.points[edge1_idx].forward_edges[edge1_edge_idx].following_edge_idx;
        let edge2_following_idx = self.points[edge2_idx].forward_edges[edge2_edge_idx].following_edge_idx;

        // List of edges we've added to the collision point (in the form of the edge that's divided, the position it was divided at and the index on the collision point)
        let mut new_edges       = vec![];

        // The 'b' edges both extend from our mid-point to the existing end point (provided t < 1.0)
        if !Self::t_is_one(t1) && !Self::t_is_zero(t1) {
            // If t1 is zero or one, we're not subdividing edge1
            // If zero, we're just adding the existing edge again to the collision point (so we do nothing)
            let new_following_idx = self.points[collision_point].forward_edges.len();

            new_edges.push((edge1_idx, edge1_edge_idx, t1, new_following_idx));
            self.points[collision_point].forward_edges.push(GraphPathEdge::new(edge1_kind, edge1b.control_points(), edge1_end_idx, edge1_label, edge1_following_idx));

            // Update edge1
            self.points[edge1_idx].forward_edges[edge1_edge_idx].set_control_points(edge1a.control_points(), collision_point, new_following_idx);

            // If t1 is zero, we're not subdividing edge1
            // If t1 is one this should leave the edge alone
            // If t1 is not one, then the previous step will have added the remaining part of
            // edge1 to the collision point
        }

        collisions.check_consistency(self);

        if !Self::t_is_one(t2) && !Self::t_is_zero(t2) {
            // If t2 is zero or one, we're not subdividing edge2
            // If zero, we're just adding the existing edge again to the collision point (so we do nothing)
            let new_following_idx = self.points[collision_point].forward_edges.len();

            new_edges.push((edge2_idx, edge2_edge_idx, t2, new_following_idx));
            self.points[collision_point].forward_edges.push(GraphPathEdge::new(edge2_kind, edge2b.control_points(), edge2_end_idx, edge2_label, edge2_following_idx));

            // Update edge2
            self.points[edge2_idx].forward_edges[edge2_edge_idx].set_control_points(edge2a.control_points(), collision_point, new_following_idx);
        }

        // The source and target edges will be divided at the midpoint: update any future collisions to take account of that
        for (point_idx, edge_idx, t, new_edge_idx) in new_edges {
            collisions.move_after_midpoint(self, collision_point, point_idx, edge_idx, t, new_edge_idx);
        }

        collisions.check_consistency(self);

        if !Self::t_is_zero(t2) {
            // If t1 is one, this should leave the edge alone
            if Self::t_is_one(t2) {
                // If t2 is one, this will have redirected the end point of t2 to the collision point: we need to move all of the edges
                let edge_idx_offset = self.points[collision_point].forward_edges.len();

                let mut edge2_end_edges = vec![];
                mem::swap(&mut self.points[edge2_end_idx].forward_edges, &mut edge2_end_edges);
                self.points[collision_point].forward_edges.extend(edge2_end_edges);

                collisions.move_all_edges(edge2_end_idx, collision_point, edge_idx_offset);
                collisions.check_consistency(self);
            }
        }
        
        if Self::t_is_zero(t2) && collision_point != edge2_idx {
            // If t2 is zero and the collision point is not the start of edge2, then edge2 should start at the collision point instead of where it does now

            // All edges that previously went to the end point now go to the collision point (and move the edges that come from that point)
            let mut next_follow_edge_idx = self.points[collision_point].forward_edges.len();
            for point in self.points.iter_mut() {
                for edge in point.forward_edges.iter_mut() {
                    if edge.end_idx == edge2_idx {
                        edge.end_idx            = collision_point;
                        edge.following_edge_idx = next_follow_edge_idx;

                        next_follow_edge_idx += 1;
                    }
                }
            }

            // All edges that currently come from edge2 need to be moved to the collision point
            let edge_idx_offset = self.points[collision_point].forward_edges.len();

            let mut edge2_edges = vec![];
            mem::swap(&mut self.points[edge2_idx].forward_edges, &mut edge2_edges);
            self.points[collision_point].forward_edges.extend(edge2_edges);

            collisions.move_all_edges(edge2_idx, collision_point, edge_idx_offset);
            collisions.check_consistency(self);
        }

        Some(collision_point)
    }

    ///
    /// Searches two ranges of points in this object and detects collisions between them, subdividing the edges
    /// and creating branch points at the appropriate places.
    /// 
    /// collide_from must indicate indices lower than collide_to
    /// 
    fn detect_collisions(&mut self, collide_from: Range<usize>, collide_to: Range<usize>, accuracy: f64) {
        // Vector of all of the collisions found in the graph
        let mut collisions = CollisionList::new();

        // TODO: for complicated paths, maybe some pre-processing for bounding boxes to eliminate trivial cases would be beneficial for performance

        // Iterate through the edges in the 'from' range
        for src_idx in collide_from {
            for src_edge_idx in 0..self.points[src_idx].forward_edges.len() {
                // Only visit target points that have not already been visited as a source point (assume that collide_to is always a higher range than collide_from)
                let tgt_start   = collide_to.start.max(src_idx+1);
                let tgt_end     = collide_to.end.max(src_idx+1);
                let collide_to  = tgt_start..tgt_end;

                // Compare to each point in the collide_to range
                for tgt_idx in collide_to.into_iter() {
                    for tgt_edge_idx in 0..self.points[tgt_idx].forward_edges.len() {
                        // Don't collide edges against themselves
                        if src_idx == tgt_idx && src_edge_idx == tgt_edge_idx { continue; }

                        // Create edge objects for each side
                        let src_curve               = GraphEdge::new(self, GraphEdgeRef { start_idx: src_idx, edge_idx: src_edge_idx, reverse: false });
                        let tgt_curve               = GraphEdge::new(self, GraphEdgeRef { start_idx: tgt_idx, edge_idx: tgt_edge_idx, reverse: false });

                        // Quickly reject edges with non-overlapping bounding boxes
                        let src_edge_bounds         = src_curve.fast_bounding_box::<Bounds<_>>();
                        let tgt_edge_bounds         = tgt_curve.fast_bounding_box::<Bounds<_>>();
                        if !src_edge_bounds.overlaps(&tgt_edge_bounds) { continue; }

                        // Find the collisions between these two edges
                        let mut curve_collisions    = curve_intersects_curve_clip(&src_curve, &tgt_curve, accuracy);

                        // Remove any pairs of collisions that are too close together
                        remove_and_round_close_collisions(&mut curve_collisions, &src_curve, &tgt_curve);

                        // The are the points we need to divide the existing edges at and add branches
                        let tgt_idx = tgt_idx;
                        for (src_t, tgt_t) in curve_collisions {
                            // A collision at t=1 is the same as a collision on t=0 on a following edge
                            // Edge doesn't actually matter for these (as the ray will collide with all of the following edges)
                            let (src_idx, src_edge_idx, src_t) = if Self::t_is_one(src_t) {
                                (self.points[src_idx].forward_edges[src_edge_idx].end_idx, 0, 0.0)
                            } else {
                                (src_idx, src_edge_idx, src_t)
                            };

                            let (tgt_idx, tgt_edge_idx, tgt_t) = if Self::t_is_one(tgt_t) {
                                (self.points[tgt_idx].forward_edges[tgt_edge_idx].end_idx, 0, 0.0)
                            } else {
                                (tgt_idx, tgt_edge_idx, tgt_t)
                            };

                            debug_assert!(src_idx < self.points.len());
                            debug_assert!(tgt_idx < self.points.len());
                            debug_assert!(src_edge_idx < self.points[src_idx].forward_edges.len());
                            debug_assert!(tgt_edge_idx < self.points[tgt_idx].forward_edges.len());

                            // Add this as a collision
                            let src = Collision { idx: src_idx, edge: src_edge_idx, t: src_t };
                            let tgt = Collision { idx: tgt_idx, edge: tgt_edge_idx, t: tgt_t };
                            collisions.push((src, tgt));
                        }
                    }
                }
            }
        }

        collisions.check_consistency(self);

        // Apply the divisions to the edges
        while let Some((src, tgt)) = collisions.pop() {
            // Join the edges
            let _new_mid_point = self.join_edges_at_intersection((src.idx, src.edge), (tgt.idx, tgt.edge), src.t, tgt.t, &mut collisions);
            collisions.check_consistency(self);
        }

        // Recompute the reverse connections
        self.recalculate_reverse_connections();
    }

    ///
    /// Collides this path against another, generating a merged path
    /// 
    /// Anywhere this graph intersects the second graph, a point with two edges will be generated. All edges will be left as
    /// interior or exterior depending on how they're set on the graph they originate from.
    /// 
    /// Working out the collision points is the first step to performing path arithmetic: the resulting graph can be altered
    /// to specify edge types - knowing if an edge is an interior or exterior edge makes it possible to tell the difference
    /// between a hole cut into a shape and an intersection.
    /// 
    pub fn collide(mut self, collide_path: GraphPath<Point, Label>, accuracy: f64) -> GraphPath<Point, Label> {
        // Generate a merged path with all of the edges
        let collision_offset    = self.points.len();
        self                    = self.merge(collide_path);

        // Search for collisions between our original path and the new one
        let total_points = self.points.len();
        self.detect_collisions(0..collision_offset, collision_offset..total_points, accuracy);

        // Return the result
        self
    }

    ///
    /// Finds any collisions between existing points in the graph path
    ///
    pub fn self_collide(&mut self, accuracy: f64) {
        let total_points = self.points.len();
        self.detect_collisions(0..total_points, 0..total_points, accuracy);
    }

    ///
    /// Returns true if a curve is collinear given the set of coefficients for a ray
    ///
    #[inline]
    fn curve_is_collinear<'a>(edge: &GraphEdge<'a, Point, Label>, (a, b, c): (f64, f64, f64)) -> bool {
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
    fn ray_can_intersect<'a>(edge: &GraphEdge<'a, Point, Label>, (a, b, c): (f64, f64, f64)) -> bool {
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
                if Self::curve_is_collinear(&incoming, (a, b, c)) {
                    continue;
                }

                // Fetch the leaving edge for the incoming edge
                let leaving     = GraphEdgeRef { start_idx: point_idx, edge_idx: incoming.following_edge_idx(), reverse: false };
                let mut leaving = GraphEdge::new(self, leaving);

                // Follow the path until we complete a loop or find a leaving edge that's not collinear
                while Self::curve_is_collinear(&leaving, (a, b, c)) {
                    leaving = leaving.next_edge();

                    if leaving.start_point_index() == point_idx {
                        // Found a loop that was entirely collinear
                        // (Provided that the following edges always form a closed path this should always be reached, which is currently always true for the means we have to create a graph path)
                        break;
                    }
                }

                // If it's not colinear, add to the set of crossing edges
                if !Self::curve_is_collinear(&leaving, (a, b, c)) {
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
            .filter(move |edge| !Self::curve_is_collinear(&edge, ray_coeffs))
            .filter(move |edge| Self::ray_can_intersect(&edge, ray_coeffs))
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

        for edge in self.all_edges().filter(|edge| Self::curve_is_collinear(&edge, ray_coeffs)) {
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
                    if position.is_near_to(&edge.end_point(), SMALL_DISTANCE) && self.edges_for_point(edge.end_point_index()).any(|next| Self::curve_is_collinear(&next, ray_coeffs)) {
                        false
                    } else {
                        true
                    }
                } else if *curve_t < 0.1 {
                    let edge = GraphEdge::new(self, *collision);

                    // If any preceding edge is collinear, remove this collision
                    if position.is_near_to(&edge.start_point(), SMALL_DISTANCE) && self.reverse_edges_for_point(collision.start_idx).any(|previous| Self::curve_is_collinear(&previous, ray_coeffs)) {
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
                if Self::curve_is_collinear(&edge, ray_coeffs) {
                    let mut edge = edge;

                    loop {
                        edge = edge.next_edge();
                        if !Self::curve_is_collinear(&edge, ray_coeffs) {
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
            .filter(move |(collision, curve_t, _line_t, position)| {
                if *curve_t < 0.00001 && self.points[collision.start_idx].position.is_near_to(&position, SMALL_DISTANCE) {
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
                if *curve_t < 0.001 {
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
                let start_point = &self.points[collision.start_idx].position;
                let offset      = *start_point - position;

                if curve_t < 0.001 || offset.dot(&offset) < (SMALL_DISTANCE * SMALL_DISTANCE) {
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

        collisions.sort_by(|(_edge_a, _curve_t_a, line_t_a, _pos_a), (_edge_b, _curve_t_b, line_t_b, _pos_b)| line_t_a.partial_cmp(line_t_b).unwrap_or(Ordering::Equal));

        collisions
   }

    ///
    /// Remove any edges marked as interior
    ///
    pub fn remove_interior_edges(&mut self) {
        for point_idx in 0..(self.points.len()) {
            self.points[point_idx].forward_edges.retain(|edge| edge.kind != GraphPathEdgeKind::Interior);
        }
    }

    ///
    /// Returns the GraphEdge for an edgeref
    ///
    #[inline]
    pub fn get_edge<'a>(&'a self, edge: GraphEdgeRef) -> GraphEdge<'a, Point, Label> {
        GraphEdge::new(self, edge)
    }

    ///
    /// Sets the kind of a single edge
    ///
    #[inline]
    pub fn set_edge_kind(&mut self, edge: GraphEdgeRef, new_type: GraphPathEdgeKind) {
        self.points[edge.start_idx].forward_edges[edge.edge_idx].kind = new_type;
    }

    ///
    /// Sets the label of a single edge
    ///
    #[inline]
    pub fn set_edge_label(&mut self, edge: GraphEdgeRef, new_label: Label) {
        self.points[edge.start_idx].forward_edges[edge.edge_idx].label = new_label;
    }

    ///
    /// Returns the type of the edge pointed to by an edgeref
    ///
    #[inline]
    pub fn edge_kind(&self, edge: GraphEdgeRef) -> GraphPathEdgeKind {
        self.points[edge.start_idx].forward_edges[edge.edge_idx].kind
    }

    ///
    /// Returns the label of the edge pointed to by an edgeref
    ///
    #[inline]
    pub fn edge_label(&self, edge: GraphEdgeRef) -> Label {
        self.points[edge.start_idx].forward_edges[edge.edge_idx].label
    }

    ///
    /// Sets the kind of an edge and any connected edge where there are no intersections (only one edge)
    ///
    pub fn set_edge_kind_connected(&mut self, edge: GraphEdgeRef, kind: GraphPathEdgeKind) {
        let mut current_edge    = edge;
        let mut visited         = vec![false; self.points.len()];

        // Move forward
        loop {
            // Set the kind of the current edge
            self.set_edge_kind(current_edge, kind);
            visited[current_edge.start_idx] = true;

            // Pick the next edge
            let end_idx = self.points[current_edge.start_idx].forward_edges[current_edge.edge_idx].end_idx;
            let edges   = &self.points[end_idx].forward_edges;

            if edges.len() != 1 {
                // At an intersection
                break;
            } else {
                // Move on
                current_edge = GraphEdgeRef {
                    start_idx:  end_idx,
                    edge_idx:   0,
                    reverse:    false
                }
            }

            // Also stop if we've followed the loop all the way around
            if visited[current_edge.start_idx] {
                break;
            }
        }

        // Move backwards
        current_edge = edge;
        loop {
            // Mark this point as visited
            visited[current_edge.start_idx] = true;

            if self.points[current_edge.start_idx].connected_from.len() != 1 {
                // There is more than one incoming edge
                break;
            } else {
                // There's a single preceding edge
                let current_point_idx   = current_edge.start_idx;
                let previous_point_idx  = self.points[current_edge.start_idx].connected_from[0];

                // Find the index of the preceding edge
                let previous_edge_idx   = (0..(self.points[previous_point_idx].forward_edges.len()))
                    .into_iter()
                    .filter(|edge_idx| self.points[previous_point_idx].forward_edges[*edge_idx].end_idx == current_point_idx)
                    .nth(0)
                    .expect("Previous edge");

                // Move on to the next edge
                current_edge = GraphEdgeRef {
                    start_idx:  previous_point_idx,
                    edge_idx:   previous_edge_idx,
                    reverse:    false
                };

                // Change its kind
                self.set_edge_kind(current_edge, kind);
            }

            // Also stop if we've followed the loop all the way around
            if visited[current_edge.start_idx] {
                break;
            }
        }
    }

    ///
    /// Finds the exterior edges and turns them into a series of paths
    ///
    pub fn exterior_paths<POut: BezierPathFactory<Point=Point>>(&self) -> Vec<POut> {
        // List of paths returned by this function
        let mut exterior_paths = vec![];

        // Array of points visited on a path that we've added to the result
        let mut visited = vec![false; self.points.len()];

        for point_idx in 0..(self.points.len()) {
            // Ignore this point if we've already visited it as part of a path
            if visited[point_idx] {
                continue;
            }

            // Use Dijkstra's algorithm to search for the shortest path that returns to point_idx
            // This allows for loops or other constructs to exist within the edges, which can happen with sufficiently complicated arithmetic operations
            // The result will sometimes be incorrect for these situations.
            // (Ideally we'd try to find a path that visits some points multiple times when this happens)
            let mut previous_point  = vec![None; self.points.len()];
            let mut points_to_check = vec![(point_idx, point_idx)];

            // Loop until we find a previous point for the initial point (indicating we've got a loop of points)
            while previous_point[point_idx].is_none() {
                if points_to_check.len() == 0 {
                    // Ran out of points to check to find a loop (there is no loop for this point)
                    break;
                }

                let mut next_points_to_check = vec![];

                // Check all of the points we found last time (ie, breadth-first search of the graph)
                for (previous_point_idx, current_point_idx) in points_to_check {
                    let edges = if current_point_idx == point_idx {
                        // For the first point, only search forward
                        self.reverse_edges_for_point(current_point_idx).collect::<Vec<_>>()
                    } else {
                        // For all other points, search all edges
                        self.edges_for_point(current_point_idx)
                            .chain(self.reverse_edges_for_point(current_point_idx))
                            .collect::<Vec<_>>()
                    };

                    // Follow the edges for this point
                    for edge in edges {
                        // Only following exterior edges
                        if edge.kind() != GraphPathEdgeKind::Exterior {
                            continue;
                        }

                        // Find the point that this edge goes to
                        let next_point_idx = edge.end_point_index();

                        if previous_point[next_point_idx].is_some() {
                            // We've already visited this point
                            continue;
                        }

                        if next_point_idx == previous_point_idx {
                            // This edge is going backwards around the graph
                            continue;
                        }

                        // Record the current point as the previous point for the end point of this edge
                        previous_point[next_point_idx] = Some((current_point_idx, edge));

                        // Check the edges connected to this point next
                        next_points_to_check.push((current_point_idx, next_point_idx));
                    }
                }

                // Check the set of points we found during this run through the loop next time
                points_to_check = next_points_to_check;
            }

            // If we found a loop, generate a path
            if previous_point[point_idx].is_some() {
                let mut path_points     = vec![];
                let mut cur_point_idx   = point_idx;

                while let Some((last_point_idx, ref edge)) = previous_point[cur_point_idx] {
                    // Push to the path points (we're following the edges in reverse, so points are in reverse order)
                    let (cp1, cp2)  = edge.control_points();
                    let start_point = edge.start_point();

                    path_points.push((cp2, cp1, start_point));

                    // Mark this point as visited so we don't try to include it in a future path
                    visited[last_point_idx] = true;

                    // Move back along the path
                    cur_point_idx = last_point_idx;

                    if cur_point_idx == point_idx {
                        // Finished the loop
                        break;
                    }
                }

                // Start point of the path is the initial point we checked
                let start_point = self.points[point_idx].position.clone();

                let new_path    = POut::from_points(start_point, path_points);
                exterior_paths.push(new_path);
            }
        }

        // Return the set of exterior paths
        exterior_paths
    }
}

///
/// Represents an edge in a graph path
/// 
#[derive(Clone)]
pub struct GraphEdge<'a, Point: 'a, Label: 'a> {
    /// The graph that this point is for
    graph: &'a GraphPath<Point, Label>,

    /// A reference to the edge this point is for
    edge: GraphEdgeRef
}

impl<'a, Point: 'a, Label: 'a+Copy> GraphEdge<'a, Point, Label> {
    ///
    /// Creates a new graph edge (with an edge kind of 'exterior')
    /// 
    #[inline]
    fn new(graph: &'a GraphPath<Point, Label>, edge: GraphEdgeRef) -> GraphEdge<'a, Point, Label> {
        debug_assert!(edge.start_idx < graph.points.len());
        debug_assert!(edge.edge_idx < graph.points[edge.start_idx].forward_edges.len());

        GraphEdge {
            graph:  graph,
            edge:   edge
        }
    }

    ///
    /// Returns true if this edge is going backwards around the path
    ///
    #[inline]
    pub fn is_reversed(&self) -> bool {
        self.edge.reverse
    }

    ///
    /// Returns a reversed version of this edge
    ///
    #[inline]
    fn reversed(mut self) -> Self {
        self.edge.reverse = !self.edge.reverse;
        self
    }

    ///
    /// Following the existing path, returns the next edge index
    ///
    fn next_edge(mut self) -> Self {
        let next_point_idx  = self.end_point_index();
        let next_edge_idx   = self.following_edge_idx();

        self.edge.start_idx = next_point_idx;
        self.edge.edge_idx  = next_edge_idx;

        self
    }

    ///
    /// Returns the following edge index for this edge (provided it's not reversed)
    ///
    #[inline]
    fn following_edge_idx(&self) -> usize {
        if !self.edge.reverse {
            self.graph.points[self.edge.start_idx].forward_edges[self.edge.edge_idx].following_edge_idx
        } else {
            unimplemented!()
        }
    }

    ///
    /// Retrieves a reference to the edge in the graph
    ///
    #[inline]
    fn edge<'b>(&'b self) -> &'b GraphPathEdge<Point, Label> {
        &self.graph.points[self.edge.start_idx].forward_edges[self.edge.edge_idx]
    }

    ///
    /// Returns if this is an interior or an exterior edge in the path
    /// 
    pub fn kind(&self) -> GraphPathEdgeKind {
        self.edge().kind
    }

    ///
    /// Returns the index of the start point of this edge
    /// 
    #[inline]
    pub fn start_point_index(&self) -> usize {
        if self.edge.reverse {
            self.edge().end_idx
        } else {
            self.edge.start_idx
        }
    }

    ///
    /// Returns the index of the end point of this edge
    /// 
    #[inline]
    pub fn end_point_index(&self) -> usize {
        if self.edge.reverse {
            self.edge.start_idx
        } else {
            self.edge().end_idx
        }
    }

    ///
    /// The label attached to this edge
    ///
    #[inline]
    pub fn label(&self) -> Label {
        self.edge().label
    }
}

impl<'a, Point: 'a+Coordinate, Label: 'a> Geo for GraphEdge<'a, Point, Label> {
    type Point = Point;
}

impl<'a, Point: 'a+Coordinate, Label: 'a+Copy> BezierCurve for GraphEdge<'a, Point, Label> {
    ///
    /// The start point of this curve
    /// 
    #[inline]
    fn start_point(&self) -> Self::Point {
        self.graph.points[self.start_point_index()].position.clone()
    }

    ///
    /// The end point of this curve
    /// 
    #[inline]
    fn end_point(&self) -> Self::Point {
        self.graph.points[self.end_point_index()].position.clone()
    }

    ///
    /// The control points in this curve
    /// 
    #[inline]
    fn control_points(&self) -> (Self::Point, Self::Point) {
        let edge = self.edge();

        if self.edge.reverse {
            (edge.cp2.clone(), edge.cp1.clone())
        } else {
            (edge.cp1.clone(), edge.cp2.clone())
        }
    }
}

///
/// A GraphEdgeRef can be created from a GraphEdge in order to release the borrow
///
impl<'a, Point: 'a+Coordinate, Label: 'a+Copy> From<GraphEdge<'a, Point, Label>> for GraphEdgeRef {
    fn from(edge: GraphEdge<'a, Point, Label>) -> GraphEdgeRef {
        edge.edge
    }
}

///
/// A GraphEdgeRef can be created from a GraphEdge in order to release the borrow
///
impl<'a, 'b, Point: 'a+Coordinate, Label: 'a+Copy> From<&'b GraphEdge<'a, Point, Label>> for GraphEdgeRef {
    fn from(edge: &'b GraphEdge<'a, Point, Label>) -> GraphEdgeRef {
        edge.edge
    }
}

impl<'a, Point: fmt::Debug, Label: 'a+Copy> fmt::Debug for GraphEdge<'a, Point, Label> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {:?} -> {:?} ({:?} -> {:?} ({:?}, {:?}))", self.kind(), self.edge.start_idx, self.edge().end_idx, self.graph.points[self.edge.start_idx].position, self.graph.points[self.edge().end_idx].position, self.edge().cp1, self.edge().cp2)
    }
}

impl<Point: Coordinate2D+Coordinate+fmt::Debug, Label: Copy> fmt::Debug for GraphPath<Point, Label> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for point_idx in 0..(self.points.len()) {
            write!(f, "\nPoint {:?}:", point_idx)?;

            for edge in self.edges_for_point(point_idx) {
                write!(f, "\n  {:?}", edge)?;
            }
        }

        Ok(())
    }
}

///
/// Removes any pairs of collisions that are closer than `CLOSE_DISTANCE` apart, and also rounds the 
/// first and last collisions to 0.0 and 1.0
/// 
/// When colliding two bezier curves we want to avoid subdividing excessively to produce very small 
/// sections as they have a tendency to produce extra collisions due to floating point or root finding
/// errors.
///
fn remove_and_round_close_collisions<P: Coordinate+Coordinate2D, C: BezierCurve<Point=P>>(collisions: &mut Vec<(f64, f64)>, src: &C, tgt: &C) {
    let close_distance_sq = CLOSE_DISTANCE * CLOSE_DISTANCE;

    // Nothing to do if there are no collisions
    if collisions.len() == 0 {
        return;
    }

    // Work out the positions of each point
    let mut positions = collisions.iter().map(|(t1, _t2)| src.point_at_pos(*t1)).collect::<Vec<_>>();

    // Find any pairs of points that are too close together
    let mut collision_idx = 0;
    while collision_idx+1 < collisions.len() {
        // Work out the distance between this collision and the next
        let offset      = positions[collision_idx] - positions[collision_idx+1];
        let distance_sq = offset.dot(&offset);

        // Just remove both of these if they are too close together (as each collision crosses the curve once, removing collisions in pairs means that there'll still be at least one collision left if the curves actually end up crossing over)
        if distance_sq < close_distance_sq {
            collisions.remove(collision_idx); positions.remove(collision_idx);
            collisions.remove(collision_idx); positions.remove(collision_idx);
        } else {
            collision_idx += 1;
        }
    }
    
    // If the first point or the last point is close to the end of the source or target curve, clip to 0 or 1
    if collisions.len() > 0 {
        // Get the start/end points of the source and target
        let src_start   = src.start_point();
        let src_end     = src.end_point();
        let tgt_start   = tgt.start_point();
        let tgt_end     = tgt.end_point();

        // Snap collisions to 0.0 or 1.0 if they're very close to the start or end of either curve
        for collision_idx in 0..collisions.len() {
            // Snap the source side
            if collisions[collision_idx].0 > 0.0 && collisions[collision_idx].0 < 1.0 {
                let start_offset = src_start - positions[collision_idx];
                if start_offset.dot(&start_offset) < close_distance_sq {
                    collisions[collision_idx].0 = 0.0;
                }

                let end_offset = src_end - positions[collision_idx];
                if end_offset.dot(&end_offset) < close_distance_sq {
                    collisions[collision_idx].0 = 1.0;
                }
            }

            // Snap the target side
            if collisions[collision_idx].1 > 0.0 && collisions[collision_idx].1 < 1.0 {
                let start_offset = tgt_start - positions[collision_idx];
                if start_offset.dot(&start_offset) < close_distance_sq {
                    collisions[collision_idx].1 = 0.0;
                }

                let end_offset = tgt_end - positions[collision_idx];
                if end_offset.dot(&end_offset) < close_distance_sq {
                    collisions[collision_idx].1 = 1.0;
                }
            }
        }
    }
}

impl GraphRayCollision {
    ///
    /// Returns true if this collision is at an intersection
    ///
    #[inline]
    pub fn is_intersection(&self) -> bool {
        match self {
            GraphRayCollision::SingleEdge(_)        => false,
            GraphRayCollision::Intersection(_edges) => true
        }
    }

    ///
    /// Returns the edge this collision is for
    ///
    #[inline]
    pub fn edge(&self) -> GraphEdgeRef {
        match self {
            GraphRayCollision::SingleEdge(edge)     => *edge,
            GraphRayCollision::Intersection(edge)   => *edge,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;
    use super::super::super::normal::*;
    use super::super::super::super::arc::*;

    fn donut() -> GraphPath<Coord2, ()> {
        let circle1         = Circle::new(Coord2(5.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
        let inner_circle1   = Circle::new(Coord2(5.0, 5.0), 3.9).to_path::<SimpleBezierPath>();
        let circle2         = Circle::new(Coord2(9.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
        let inner_circle2   = Circle::new(Coord2(9.0, 5.0), 3.9).to_path::<SimpleBezierPath>();

        let mut circle1     = GraphPath::from_path(&circle1, ());
        circle1             = circle1.merge(GraphPath::from_path(&inner_circle1, ()));
        let mut circle2     = GraphPath::from_path(&circle2, ());
        circle2             = circle2.merge(GraphPath::from_path(&inner_circle2, ()));

        circle1.collide(circle2, 0.1)
    }

    fn tricky_path1() -> SimpleBezierPath {
        BezierPathBuilder::<SimpleBezierPath>::start(Coord2(266.4305, 634.9583))
            .curve_to((Coord2(267.89352, 634.96545), Coord2(276.2691, 647.3115)), Coord2(283.95255, 660.0379))
            .curve_to((Coord2(287.94046, 666.35474), Coord2(291.91766, 672.60645)), Coord2(295.15033, 677.43414))
            .curve_to((Coord2(296.7672, 679.91516), Coord2(298.1211, 681.9124)), Coord2(299.32123, 683.47577))
            .curve_to((Coord2(299.95978, 684.32623), Coord2(300.40076, 684.9176)), Coord2(300.98044, 685.51074))
            .curve_to((Coord2(301.33307, 685.8545), Coord2(301.51462, 686.0718)), Coord2(301.92783, 686.3648))
            .curve_to((Coord2(302.63144, 686.6535), Coord2(302.6845, 686.9835)), Coord2(303.79065, 687.13))
            .curve_to((Coord2(308.23322, 698.75146), Coord2(314.235, 706.79364)), Coord2(320.5527, 711.571))
            .curve_to((Coord2(323.84628, 713.9084), Coord2(326.7522, 715.38696)), Coord2(329.93036, 715.9504))
            .curve_to((Coord2(333.10065, 716.4182), Coord2(336.06982, 716.2095)), Coord2(338.80997, 715.17615))
            .curve_to((Coord2(344.1068, 713.1569), Coord2(348.558, 708.8886)), Coord2(352.09903, 704.2416))
            .curve_to((Coord2(355.6339, 699.64606), Coord2(358.63943, 694.3838)), Coord2(361.0284, 690.2511))
            .curve_to((Coord2(352.29608, 691.48425), Coord2(348.7531, 697.58563)), Coord2(344.9467, 702.02875))
            .curve_to((Coord2(343.1644, 704.2118), Coord2(340.9616, 706.1748)), Coord2(338.98895, 707.4077))
            .curve_to((Coord2(337.17404, 708.7338), Coord2(334.93362, 709.2896)), Coord2(332.94815, 709.3193))
            .curve_to((Coord2(338.20477, 716.0944), Coord2(342.99326, 713.658)), Coord2(346.69864, 710.2048))
            .curve_to((Coord2(350.41446, 706.8076), Coord2(353.61026, 702.4266)), Coord2(356.28525, 698.20306))
            .curve_to((Coord2(358.8071, 690.86554), Coord2(368.403, 680.78076)), Coord2(364.57346, 683.4333))
            .curve_to((Coord2(370.74402, 683.10126), Coord2(380.93408, 677.46747)), Coord2(391.3346, 669.7194))
            .curve_to((Coord2(401.82745, 661.6356), Coord2(411.92975, 652.304)), Coord2(416.44824, 642.7813))
            .curve_to((Coord2(421.56387, 630.7548), Coord2(419.29, 605.44073)), Coord2(418.97845, 598.63885))
            .curve_to((Coord2(416.0324, 600.9351), Coord2(416.06793, 605.21173)), Coord2(415.80798, 610.2456))
            .curve_to((Coord2(418.3617, 603.8127), Coord2(419.7235, 595.5345)), Coord2(417.99966, 597.9464))
            .curve_to((Coord2(417.83536, 597.29565), Coord2(417.6163, 596.428)), Coord2(417.452, 595.7772))
            .curve_to((Coord2(415.13226, 598.33954), Coord2(417.1024, 601.5625)), Coord2(415.80798, 610.2456))
            .curve_to((Coord2(419.39615, 605.133), Coord2(419.15756, 600.892)), Coord2(418.97845, 598.63885))
            .curve_to((Coord2(415.9, 605.6454), Coord2(416.15115, 630.697)), Coord2(410.98987, 640.1752))
            .curve_to((Coord2(407.398, 647.65436), Coord2(397.31293, 657.55756)), Coord2(387.45657, 664.45013))
            .curve_to((Coord2(377.50784, 671.67847), Coord2(367.18683, 676.76263)), Coord2(364.60056, 676.3969))
            .curve_to((Coord2(356.0477, 679.03125), Coord2(358.2825, 685.37573)), Coord2(350.3949, 694.47205))
            .curve_to((Coord2(347.86517, 698.46545), Coord2(345.09418, 702.3025)), Coord2(342.02982, 705.0691))
            .curve_to((Coord2(338.955, 707.7797), Coord2(336.14987, 709.45294)), Coord2(332.94815, 709.3193))
            .curve_to((Coord2(336.5865, 716.2577), Coord2(339.58755, 714.99677)), Coord2(342.64694, 713.29364))
            .curve_to((Coord2(345.54865, 711.4972), Coord2(347.85297, 709.2183)), Coord2(350.22574, 706.551))
            .curve_to((Coord2(354.72943, 701.2933), Coord2(358.0882, 695.26)), Coord2(361.0284, 690.2511))
            .curve_to((Coord2(352.55414, 690.95703), Coord2(349.8117, 695.7842)), Coord2(346.5798, 700.0057))
            .curve_to((Coord2(343.354, 704.1756), Coord2(340.01953, 707.4518)), Coord2(336.43625, 708.6749))
            .curve_to((Coord2(334.73633, 709.2627), Coord2(332.9918, 709.5996)), Coord2(331.1653, 709.1589))
            .curve_to((Coord2(329.34668, 708.8136), Coord2(326.97275, 707.9294)), Coord2(324.69394, 706.071))
            .curve_to((Coord2(319.86685, 702.45667), Coord2(313.55374, 694.77545)), Coord2(307.1513, 682.14154))
            .curve_to((Coord2(305.31448, 680.437), Coord2(305.08902, 680.6507)), Coord2(305.46603, 680.73413))
            .curve_to((Coord2(305.55258, 680.8219), Coord2(305.35938, 680.745)), Coord2(305.29236, 680.7117))
            .curve_to((Coord2(305.03268, 680.5507), Coord2(304.45453, 680.05615)), Coord2(303.91962, 679.53674))
            .curve_to((Coord2(302.7728, 678.36035), Coord2(301.16226, 676.48175)), Coord2(299.40033, 674.3327))
            .curve_to((Coord2(295.8753, 669.90015), Coord2(291.43716, 663.8746)), Coord2(286.9764, 657.9508))
            .curve_to((Coord2(277.76248, 646.196), Coord2(269.10742, 634.2079)), Coord2(266.40128, 634.45917))
            .curve_to((Coord2(266.42087, 634.7936), Coord2(266.41122, 634.6289)), Coord2(266.4305, 634.9583))
            .build()
    }

    fn overlapping_rectangle() -> SimpleBezierPath {
        BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
            .line_to(Coord2(5.0, 1.0))
            .line_to(Coord2(5.0, 3.0))
            .line_to(Coord2(7.0, 5.0))
            .line_to(Coord2(5.0, 7.0))
            .line_to(Coord2(3.0, 5.0))
            .line_to(Coord2(1.0, 5.0))
            .line_to(Coord2(5.0, 1.0))
            .line_to(Coord2(5.0, 5.0))
            .line_to(Coord2(1.0, 5.0))
            .line_to(Coord2(1.0, 1.0))
            .build()
    }

    fn looped_rectangle() -> SimpleBezierPath {
        BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))

            //.line_to(Coord2(2.0, 1.0))
            .line_to(Coord2(3.0, 1.0))
            .line_to(Coord2(3.0, 5.0))
            .line_to(Coord2(2.0, 5.0))
            .line_to(Coord2(2.0, 1.0))
            .line_to(Coord2(3.0, 1.0))

            .line_to(Coord2(5.0, 1.0))
            .line_to(Coord2(5.0, 5.0))

            //.line_to(Coord2(3.0, 5.0))
            .line_to(Coord2(2.0, 5.0))
            .line_to(Coord2(2.0, 1.0))
            .line_to(Coord2(3.0, 1.0))
            .line_to(Coord2(3.0, 5.0))
            .line_to(Coord2(2.0, 5.0))

            .line_to(Coord2(1.0, 5.0))
            .line_to(Coord2(1.0, 1.0))
            .build()
    }

    #[test]
    fn raw_donut_collisions() {
        let donut = donut();

        let raw_collisions = donut.raw_ray_collisions(&(Coord2(7.000584357101389, 8.342524209216537), Coord2(6.941479643691172, 8.441210096108172)));
        println!("{:?}", raw_collisions.collect::<Vec<_>>());

        // assert!(false);
    }

    #[test]
    fn collinear_collision_along_convex_edge_produces_no_collisions() {
        // Just one rectangle
        let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
            .line_to(Coord2(5.0, 1.0))
            .line_to(Coord2(5.0, 5.0))
            .line_to(Coord2(1.0, 5.0))
            .line_to(Coord2(1.0, 1.0))
            .build();

        // Collide along the vertical seam of this graph
        let gp = GraphPath::from_path(&rectangle1, ());

        let collisions = gp.collinear_ray_collisions(&(Coord2(5.0, 0.0), Coord2(5.0, 5.0)))
            .collect::<Vec<_>>();
        assert!(collisions.len() == 0);
    }

    #[test]
    fn raw_collision_along_convex_edge_produces_no_collisions() {
        // Just one rectangle
        let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
            .line_to(Coord2(5.0, 1.0))
            .line_to(Coord2(5.0, 5.0))
            .line_to(Coord2(1.0, 5.0))
            .line_to(Coord2(1.0, 1.0))
            .build();

        // Collide along the vertical seam of this graph
        let gp = GraphPath::from_path(&rectangle1, ());

        let collisions = gp.raw_ray_collisions(&(Coord2(5.0, 0.0), Coord2(5.0, 5.0)));
        let collisions = gp.remove_collisions_before_or_after_collinear_section(&(Coord2(5.0, 0.0), Coord2(5.0, 5.0)), collisions);
        let collisions = collisions.collect::<Vec<_>>();

        assert!(collisions.len() == 0);
    }

    #[test]
    fn collinear_collision_along_concave_edge_produces_single_collision() {
        let concave_shape = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
            .line_to(Coord2(5.0, 1.0))
            .line_to(Coord2(5.0, 5.0))
            .line_to(Coord2(6.0, 7.0))
            .line_to(Coord2(3.0, 7.0))
            .line_to(Coord2(1.0, 5.0))
            .line_to(Coord2(1.0, 1.0))
            .build();

        // Collide along the vertical seam of this graph
        let gp  = GraphPath::from_path(&concave_shape, ());
        let ray = (Coord2(5.0, 0.0), Coord2(5.0, 5.0));

        let collisions = gp.collinear_ray_collisions(&ray);
        let collisions = collisions.collect::<Vec<_>>();

        assert!(collisions.len() == 1);
    }

    #[test]
    fn raw_collision_along_concave_edge_produces_single_collision() {
        let concave_shape = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
            .line_to(Coord2(5.0, 1.0))
            .line_to(Coord2(5.0, 5.0))
            .line_to(Coord2(6.0, 7.0))
            .line_to(Coord2(3.0, 7.0))
            .line_to(Coord2(1.0, 5.0))
            .line_to(Coord2(1.0, 1.0))
            .build();

        // Collide along the vertical seam of this graph
        let gp  = GraphPath::from_path(&concave_shape, ());
        let ray = (Coord2(5.0, 0.0), Coord2(5.0, 5.0));

        let collisions = gp.raw_ray_collisions(&ray);
        let collisions = gp.remove_collisions_before_or_after_collinear_section(&(Coord2(5.0, 0.0), Coord2(5.0, 5.0)), collisions);
        let collisions = collisions.collect::<Vec<_>>();

        assert!(collisions.len() == 1);
    }

    #[test]
    fn concave_collision_breakdown() {
        let concave_shape = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
            .line_to(Coord2(5.0, 1.0))
            .line_to(Coord2(5.0, 5.0))
            .line_to(Coord2(6.0, 7.0))
            .line_to(Coord2(3.0, 7.0))
            .line_to(Coord2(1.0, 5.0))
            .line_to(Coord2(1.0, 1.0))
            .build();

        // Collide along the vertical seam of this graph
        let gp  = GraphPath::from_path(&concave_shape, ());
        let ray = (Coord2(5.0, 0.0), Coord2(5.0, 5.0));

        // Raw collisions
        let collinear_collisions    = gp.collinear_ray_collisions(&ray).collect::<Vec<_>>();
        let normal_collisions       = gp.raw_ray_collisions(&ray).collect::<Vec<_>>();
        let normal_collisions       = gp.remove_collisions_before_or_after_collinear_section(&ray, normal_collisions).collect::<Vec<_>>();

        assert!(collinear_collisions.len() == 1);
        assert!(normal_collisions.len() == 1);

        // Chain them together
        let collisions = collinear_collisions.into_iter().chain(normal_collisions.into_iter()).collect::<Vec<_>>();
        assert!(collisions.len() == 2);

        // Filter for accuracy
        let collisions = gp.move_collisions_at_end_to_beginning(collisions).collect::<Vec<_>>();
        assert!(collisions.len() == 2);
        let collisions = gp.move_collinear_collisions_to_end(&ray, collisions).collect::<Vec<_>>();
        assert!(collisions.len() == 2);
        let collisions = gp.remove_glancing_collisions(&ray, collisions).collect::<Vec<_>>();
        assert!(collisions.len() == 2);
        let collisions = gp.remove_duplicate_collisions_at_start(collisions).collect::<Vec<_>>();
        assert!(collisions.len() == 2);
        let collisions = gp.flag_collisions_at_intersections(collisions).collect::<Vec<_>>();
        assert!(collisions.len() == 2);
    }

    #[test]
    fn interior_point_produces_two_collisions() {
        let with_interior_point = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
            .line_to(Coord2(5.0, 1.0))
            .line_to(Coord2(5.0, 5.0))
            .line_to(Coord2(2.0, 2.0))
            .line_to(Coord2(4.0, 2.0))
            .line_to(Coord2(1.0, 5.0))
            .line_to(Coord2(1.0, 1.0))
            .build();

        let mut with_interior_point = GraphPath::from_path(&with_interior_point, ());
        with_interior_point.self_collide(0.01);

        let ray         = (Coord2(0.0, 3.0), Coord2(1.0, 3.0));
        let collisions  = with_interior_point.raw_ray_collisions(&ray);

        let collisions = collisions.collect::<Vec<_>>();
        println!("{:?}", with_interior_point);
        println!("{:?}", collisions);

        assert!(collisions.len() == 4);

        // Filter for accuracy
        let collisions = with_interior_point.move_collisions_at_end_to_beginning(collisions).collect::<Vec<_>>();
        assert!(collisions.len() == 4);
        let collisions = with_interior_point.move_collinear_collisions_to_end(&ray, collisions).collect::<Vec<_>>();
        assert!(collisions.len() == 4);
        let collisions = with_interior_point.remove_glancing_collisions(&ray, collisions).collect::<Vec<_>>();
        assert!(collisions.len() == 4);
        println!("{:?}", collisions);
        let collisions = with_interior_point.remove_duplicate_collisions_at_start(collisions).collect::<Vec<_>>();
        assert!(collisions.len() == 4);
        let collisions = with_interior_point.flag_collisions_at_intersections(collisions).collect::<Vec<_>>();
        assert!(collisions.len() == 4);
    }

    #[test]
    fn ray_cast_with_tricky_path_after_self_collide() {
        let tricky      = tricky_path1();
        let mut tricky  = GraphPath::from_path(&tricky, ());

        tricky.self_collide(0.01);

        for edge in tricky.all_edges() {
            let target  = edge.point_at_pos(0.5);
            let normal  = edge.normal_at_pos(0.5);
            let ray     = (target, target+normal);

            let collisions = tricky.ray_collisions(&ray);

            // Should be an even number of collisions
            assert!((collisions.len()&1) == 0);
        }
    }

    #[test]
    fn single_difficult_ray_cast_with_tricky_path_before_self_collide() {
        let tricky      = tricky_path1();
        let tricky      = GraphPath::from_path(&tricky, ());

        let ray         = (Coord2(344.7127586558301, 702.311674360346), Coord2(344.6914625870749, 702.2935114955856));
        let collisions  = tricky.ray_collisions(&ray);

        println!("{:?}", tricky);
        println!("{:?}", collisions);
        assert!((collisions.len()&1) == 0);
    }

    #[test]
    fn single_difficult_ray_cast_with_tricky_path_after_self_collide() {
        let tricky      = tricky_path1();
        let mut tricky  = GraphPath::from_path(&tricky, ());

        tricky.self_collide(0.01);

        let ray         = (Coord2(344.7127586558301, 702.311674360346), Coord2(344.6914625870749, 702.2935114955856));
        let collisions  = tricky.ray_collisions(&ray);

        println!("{:?}", tricky);
        println!("{:?}", collisions);
        assert!((collisions.len()&1) == 0);
    }

    #[test]
    fn overlapping_rectangle_ray_cast_after_self_collide() {
        let overlapping     = overlapping_rectangle();
        let mut overlapping = GraphPath::from_path(&overlapping, ());

        overlapping.self_collide(0.01);

        let ray         = (Coord2(3.0, 0.0), Coord2(3.0, 5.0));
        let collisions  = overlapping.ray_collisions(&ray);

        println!("{:?}", overlapping);
        println!("{:?}", collisions);
        assert!((collisions.len()&1) == 0);
    }

    #[test]
    fn looped_rectangle_ray_cast_after_self_collide() {
        let looped     = looped_rectangle();
        let mut looped = GraphPath::from_path(&looped, ());

        looped.self_collide(0.01);
        println!("{:?}", looped);

        for edge in looped.all_edges() {
            let target  = edge.point_at_pos(0.5);
            let normal  = edge.normal_at_pos(0.5);
            let ray     = (target, target+normal);

            let collisions = looped.ray_collisions(&ray);

            // Should be an even number of collisions
            assert!((collisions.len()&1) == 0);
        }
    }
}
