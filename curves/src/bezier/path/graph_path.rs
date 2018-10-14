use super::path::*;
use super::super::curve::*;
use super::super::intersection::*;
use super::super::super::geo::*;
use super::super::super::line::*;
use super::super::super::coordinate::*;

use std::fmt;
use std::mem;
use std::vec;
use std::iter;
use std::ops::Range;
use std::cmp::Ordering;

const CLOSE_DISTANCE: f64 = 0.01;

///
/// Represents a collision between a ray and a GraphPath
///
#[derive(Clone, Debug, PartialEq)]
pub enum GraphRayCollision {
    /// Collision against a single edge
    SingleEdge(GraphEdgeRef),

    /// Collision against an intersection point
    Intersection(Vec<GraphEdgeRef>)
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

    /// Index of the source path for this edge
    source_path_idx: usize,

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
    fn new(kind: GraphPathEdgeKind, (cp1, cp2): (Point, Point), end_idx: usize, label: Label, source_path_idx: usize) -> GraphPathEdge<Point, Label> {
        GraphPathEdge {
            label, kind, cp1, cp2, end_idx, source_path_idx
        }
    }

    ///
    /// Updates the control points of this edge
    /// 
    #[inline]
    fn set_control_points(&mut self, (cp1, cp2): (Point, Point), end_idx: usize) {
        self.cp1 = cp1;
        self.cp2 = cp2;
        self.end_idx = end_idx;
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
            let distance = end_point - last_point_pos;
            if distance.dot(&distance) < CLOSE_DISTANCE { 
                let cp1_distance = cp1-last_point_pos;
                let cp2_distance = cp2-cp1;

                if cp1_distance.dot(&cp1_distance) < CLOSE_DISTANCE && cp2_distance.dot(&cp2_distance) < CLOSE_DISTANCE {
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
                // Need to draw a line to the last point
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
                    edge.source_path_idx    += next_path_idx;
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
    fn t_is_zero(t: f64) -> bool { t < 0.01 }

    ///
    /// True if the t value is effective at the end of the curve
    /// 
    #[inline]
    fn t_is_one(t: f64) -> bool { t > 0.99 }

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
        let edge1_kind      = self.points[edge1_idx].forward_edges[edge1_edge_idx].kind;
        let edge2_kind      = self.points[edge2_idx].forward_edges[edge2_edge_idx].kind;
        let edge1_label     = self.points[edge1_idx].forward_edges[edge1_edge_idx].label;
        let edge2_label     = self.points[edge2_idx].forward_edges[edge2_edge_idx].label;
        let edge1_end_idx   = self.points[edge1_idx].forward_edges[edge1_edge_idx].end_idx;
        let edge2_end_idx   = self.points[edge2_idx].forward_edges[edge2_edge_idx].end_idx;
        let edge1_path_idx  = self.points[edge1_idx].forward_edges[edge1_edge_idx].source_path_idx;
        let edge2_path_idx  = self.points[edge2_idx].forward_edges[edge2_edge_idx].source_path_idx;

        // List of edges we've added to the collision point (in the form of the edge that's divided, the position it was divided at and the index on the collision point)
        let mut new_edges   = vec![];

        // The 'b' edges both extend from our mid-point to the existing end point (provided
        // t < 1.0)
        if !Self::t_is_one(t1) && !Self::t_is_zero(t1) {
            // If t1 is zero or one, we're not subdividing edge1
            // If zero, we're just adding the existing edge again to the collision point (so we do nothing)
            new_edges.push((edge1_idx, edge1_edge_idx, t1, self.points[collision_point].forward_edges.len()));
            self.points[collision_point].forward_edges.push(GraphPathEdge::new(edge1_kind, edge1b.control_points(), edge1_end_idx, edge1_label, edge1_path_idx));
        }
        if !Self::t_is_one(t2) && !Self::t_is_zero(t2) {
            // If t2 is zero or one, we're not subdividing edge2
            // If zero, we're just adding the existing edge again to the collision point (so we do nothing)
            new_edges.push((edge2_idx, edge2_edge_idx, t2, self.points[collision_point].forward_edges.len()));
            self.points[collision_point].forward_edges.push(GraphPathEdge::new(edge2_kind, edge2b.control_points(), edge2_end_idx, edge2_label, edge2_path_idx));
        }

        // The 'a' edges both update the initial edge, provided t is not 0
        if !Self::t_is_zero(t1) && !Self::t_is_one(t1) {
            self.points[edge1_idx].forward_edges[edge1_edge_idx].set_control_points(edge1a.control_points(), collision_point);

            // If t1 is zero, we're not subdividing edge1
            // If t1 is one this should leave the edge alone
            // If t1 is not one, then the previous step will have added the remaining part of
            // edge1 to the collision point
        }

        collisions.check_consistency(self);

        // The source and target edges will be divided at the midpoint: update any future collisions to take account of that
        for (point_idx, edge_idx, t, new_edge_idx) in new_edges {
            collisions.move_after_midpoint(self, collision_point, point_idx, edge_idx, t, new_edge_idx);
        }

        collisions.check_consistency(self);

        if !Self::t_is_zero(t2) {
            self.points[edge2_idx].forward_edges[edge2_edge_idx].set_control_points(edge2a.control_points(), collision_point);

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

            // All edges that previously went to the end point now go to the collision point
            for point in self.points.iter_mut() {
                for edge in point.forward_edges.iter_mut() {
                    if edge.end_idx == edge2_idx {
                        edge.end_idx = collision_point;
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

        // The points that have had collisions exactly on them (we only collide them once)
        let mut collided = vec![false; self.points.len()];

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
                        let src_curve           = GraphEdge::new(self, GraphEdgeRef { start_idx: src_idx, edge_idx: src_edge_idx, reverse: false });
                        let tgt_curve           = GraphEdge::new(self, GraphEdgeRef { start_idx: tgt_idx, edge_idx: tgt_edge_idx, reverse: false });

                        // Quickly reject edges with non-overlapping bounding boxes
                        let src_edge_bounds     = src_curve.fast_bounding_box::<Bounds<_>>();
                        let tgt_edge_bounds     = tgt_curve.fast_bounding_box::<Bounds<_>>();
                        if !src_edge_bounds.overlaps(&tgt_edge_bounds) { continue; }

                        // Find the collisions between these two edges
                        let curve_collisions    = curve_intersects_curve_clip(&src_curve, &tgt_curve, accuracy);

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

                            // Allow only one collision exactly on a point
                            if Self::t_is_zero(src_t) {
                                if collided[src_idx] { 
                                    continue;
                                } else {
                                    collided[src_idx] = true;
                                }
                            }

                            if Self::t_is_zero(tgt_t) {
                                if collided[tgt_idx] { 
                                    continue;
                                } else {
                                    collided[tgt_idx] = true;
                                }
                            }

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
    /// Returns true if a particular ray is 'grazing' the start of a particular edge. The ray is assumed to intersect the edge.
    /// 'Grazing' means that the corner at the edge is of an angle such that the ray never enters/leaves the shape by this
    /// intersection.
    ///
    fn ray_is_grazing(&self, point_idx: usize, edge_idx: usize, (a, b, c): (f64, f64, f64)) -> bool {
        // Find the previous edges
        let previous_edges = self.points[point_idx].connected_from
            .iter()
            .flat_map(|connected_from_idx| self.points[*connected_from_idx].forward_edges
                    .iter()
                    .enumerate()
                    .filter(|(_idx, edge)| edge.end_idx == point_idx)
                    .map(move |(edge_idx, _edge)| (*connected_from_idx, edge_idx)));

        for (previous_point_idx, previous_edge_idx) in previous_edges {
            // The control points on either side of the collision determine the direction of the line on either side
            let cp1     = &self.points[point_idx].forward_edges[edge_idx].cp1;
            let cp2     = &self.points[previous_point_idx].forward_edges[previous_edge_idx].cp2;

            // The ray is 'grazing' if both control points are on the same side of the collision
            // ... in the event of an intersection, if any of the incoming edges are 'grazing' (TODO: this might not work so well if it happens while trying to determine what parts of a ray are inside or outside of a shape)
            let side1 = a*cp1.x() + b*cp1.y() + c;
            let side2 = a*cp2.x() + b*cp2.y() + c;

            let side1 = if side1.abs() < 0.001 { 0.0 } else { side1.signum() };
            let side2 = if side2.abs() < 0.001 { 0.0 } else { side2.signum() };

            if side1 == side2 {
                return true;
            }
        }

        return false;
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
        if (start_point.x()*a + start_point.y()*b + c).abs() < 0.001
        && (end_point.x()*a + end_point.y()*b + c).abs() < 0.001
        && (cp1.x()*a + cp1.y()*b + c).abs() < 0.001
        && (cp2.x()*a + cp2.y()*b + c).abs() < 0.001 {
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
        // Get the list of 'incoming' edges for each path index in this graph
        let mut incoming_edge_for_path = vec![None; self.next_path_index];

        for point_idx in points.iter() {
            for incoming in self.reverse_edges_for_point(*point_idx) {
                if !Self::curve_is_collinear(&incoming, (a, b, c)) {
                    let source_path_idx                     = incoming.source_path_idx();
                    incoming_edge_for_path[source_path_idx] = Some(incoming);
                }
            }
        }

        // For every 'leaving' edge, check if they are on the other side of the path to their corresponding 'incoming' edge, and add to the result if they are
        let mut crossing_edges = vec![];

        for point_idx in points.iter() {
            for leaving in self.edges_for_point(*point_idx) {
                if let Some(incoming) = &incoming_edge_for_path[leaving.source_path_idx()] {
                    if !Self::curve_is_collinear(&leaving, (a, b, c)) {
                        let incoming_cp1    = incoming.control_points().0;
                        let leaving_cp1     = leaving.control_points().0;

                        let incoming_side   = a*incoming_cp1.x() + b*incoming_cp1.y() + c;
                        let leaving_side    = a*leaving_cp1.x() + b*leaving_cp1.y() + c;

                        if incoming_side.signum() != leaving_side.signum() {
                            // Control points are on different sides of the line, so this is a crossing edge
                            crossing_edges.push(leaving);
                        }
                    }
                }
            }
        }

        crossing_edges
    }

    ///
    /// Returns a point outside of this path
    ///
    pub fn outside_point(&self) -> Point {
        let mut outside_point = Point::origin();

        for point in self.points.iter() {
            outside_point = Point::from_smallest_components(outside_point, point.position);

            for edge in point.forward_edges.iter() {
                outside_point = Point::from_smallest_components(outside_point, edge.cp1);
                outside_point = Point::from_smallest_components(outside_point, edge.cp2);
            }
        }

        outside_point - Point::from_components(&[0.1, 0.1])
    }

    ///
    /// Finds the exterior edge (and t value) where a line first collides with this path (closest to the line
    /// start point)
    /// 
    pub fn ray_collisions<'a, L: Line<Point=Point>>(&'a self, ray: &L) -> Vec<(GraphRayCollision, f64, f64)> {
        // We'll store the result after visiting all of the edges
        let mut collision_result    = vec![];

        // Coefficients of the ray
        let ray_coeffs              = ray.coefficients();

        // List of points where we've hit the start of the line
        let mut visited_start       = vec![false; self.points.len()];

        // Search for any collinear edges
        let mut remaining_edges = vec![];

        for edge in self.all_edges() {
            if Self::curve_is_collinear(&edge, ray_coeffs) {
                // Collinear edges need to be processed in a different way. From the point of view of our ray they technically don't
                // exist, so we need to look at the edges before the collinear edge and the edges after it. If there are edges crossing
                // from one side to the other, then we need to record a collision for each. If there are no edges crossing, then we need
                // to just ignore the colinear edge

                // TODO: collinear sections may be longer than a single edge
                // TODO: the control points do provide tangent values for the curve, but they can be equal to the start point which makes this test less good
                for crossing in self.crossing_edges(ray_coeffs, vec![edge.start_point_index(), edge.end_point_index()]) {
                    let line_t  = ray.pos_for_point(crossing.start_point());
                    let curve_t = 0.0;

                    if !visited_start[crossing.start_point_index()] {
                        collision_result.push((GraphRayCollision::new(crossing.into()), curve_t, line_t));
                    }
                }

                // Mark these points as visited
                visited_start[edge.end_point_index()]   = true;
                visited_start[edge.start_point_index()] = true;
            } else {
                // Process this edge normally
                remaining_edges.push(edge);
            }
        }

        // Visit every edge in this graph and look for collisions
        for edge in remaining_edges.into_iter() {
            // If all of the points on this edge are on the same side of the ray, then it's not an intersection
            if !Self::ray_can_intersect(&edge, ray_coeffs) {
                continue;
            }

            // Find out where the line collides with this edge
            let collisions = curve_intersects_ray(&edge, ray);

            for (curve_t, line_t, _collide_pos) in collisions {
                // Collisions at the end of a curve are treated as collision on the next curve
                let (point_idx, curve_t) = if curve_t > 0.999 {
                    // Collision is at the end of the curve
                    (edge.end_point_index(), 0.0)
                } else {
                    (edge.start_point_index(), curve_t)
                };

                if Self::t_is_zero(curve_t) {
                    // Collision is at the start of the curve
                    if !visited_start[point_idx] {
                        // Mark the start of this point as visited
                        visited_start[point_idx] = true;

                        // Intersections are a single collision against multiple edges
                        let mut edges   = self.edges_for_point(point_idx)
                            .filter(|edge| !self.ray_is_grazing(edge.edge.start_idx, edge.edge.edge_idx, ray_coeffs));
                        let first_edge  = edges.next();

                        if let Some(first_edge) = first_edge {
                            let mut intersection = GraphRayCollision::new(first_edge.into());

                            for point_edge in edges {
                                intersection.push(point_edge.into());
                            }

                            collision_result.push((intersection, curve_t, line_t));
                        }
                    }
                } else {
                    // Collision is mid-way in the curve
                    collision_result.push((GraphRayCollision::new(edge.clone().into()), curve_t, line_t));
                }
            }
        }

        collision_result.sort_by(|(_edge_a, _curve_t_a, line_t_a), (_edge_b, _curve_t_b, line_t_b)| line_t_a.partial_cmp(line_t_b).unwrap_or(Ordering::Equal));
        collision_result
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
                    .unwrap();

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
        let mut exterior_paths = vec![];

        // Array of visited points
        let mut visited = vec![false; self.points.len()];

        for point_idx in 0..(self.points.len()) {
            // Ignore this point if we've already visited it as part of a path
            if visited[point_idx] {
                continue;
            }

            // Find the first exterior point
            let exterior_edge = self.edges_for_point(point_idx)
                .filter(|edge| edge.kind() == GraphPathEdgeKind::Exterior)
                .nth(0);

            if let Some(exterior_edge) = exterior_edge {
                // Follow the edge around to generate the path (we expect exterior edges to form a complete path)
                let start_point         = exterior_edge.start_point();
                let mut current_edge    = exterior_edge;
                let mut path_points     = vec![];

                loop {
                    let current_point_idx = current_edge.start_point_index();

                    // Stop once we reach a point we've already visited
                    if visited[current_point_idx] {
                        break;
                    }

                    // Mark the current point as visited
                    visited[current_point_idx] = true;

                    // Add the next edge to the path
                    let (cp1, cp2) = current_edge.control_points();
                    path_points.push((cp1, cp2, current_edge.end_point()));

                    // Find the next edge (next exterior edge in either direction that is not back the way we came)
                    let next_point_idx  = current_edge.end_point_index();
                    let next_edge       = self.edges_for_point(next_point_idx)
                        .chain(self.reverse_edges_for_point(next_point_idx))
                        .filter(|edge| edge.end_point_index() != current_point_idx)
                        .filter(|edge| edge.kind() == GraphPathEdgeKind::Exterior)
                        .nth(0);

                    if let Some(next_edge) = next_edge {
                        // Move on to the next point on this path
                        current_edge = next_edge;
                    } else {
                        // Partial path
                        // TODO: or, reversal of direction...
                        break;
                    }
                }

                // Turn into a path
                let path = POut::from_points(start_point, path_points);
                exterior_paths.push(path);
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
    /// Returns the source path index for this edge
    ///
    fn source_path_idx(&self) -> usize {
        self.graph.points[self.edge.start_idx].forward_edges[self.edge.edge_idx].source_path_idx
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

impl GraphRayCollision {
    ///
    /// Creates a new collision with a single edge
    ///
    #[inline]
    fn new(edge: GraphEdgeRef) -> GraphRayCollision {
        GraphRayCollision::SingleEdge(edge)
    }

    ///
    /// Adds a new edge to the intersection formed by this collision
    ///
    fn push(&mut self, new_edge: GraphEdgeRef) {
        use self::GraphRayCollision::*;

        match self {
            Intersection(edges) => { edges.push(new_edge); }
            SingleEdge(_)       => { 
                let edge            = mem::replace(self, Intersection(vec![]));

                if let SingleEdge(edge) = edge {
                    let intersection    = vec![edge, new_edge];
                    *self = Intersection(intersection);
                }
            }
        }
    }

    ///
    /// Returns true if this collision is at an intersection
    ///
    #[inline]
    pub fn is_intersection(&self) -> bool {
        match self {
            GraphRayCollision::SingleEdge(_)        => false,
            GraphRayCollision::Intersection(edges)  => edges.len() != 1
        }
    }
}

impl IntoIterator for GraphRayCollision {
    type Item       = GraphEdgeRef;
    type IntoIter   = GraphRayIterator;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            GraphRayCollision::SingleEdge(edge)     => GraphRayIterator::SingleEdge(iter::once(edge)),
            GraphRayCollision::Intersection(vec)    => GraphRayIterator::Intersection(vec.into_iter())
        }
    }
}

///
/// Iterator over the edges in a collision
///
pub enum GraphRayIterator {
    SingleEdge(iter::Once<GraphEdgeRef>),
    Intersection(vec::IntoIter<GraphEdgeRef>)
}

impl Iterator for GraphRayIterator {
    type Item = GraphEdgeRef;

    fn next(&mut self) -> Option<GraphEdgeRef> {
        match self {
            GraphRayIterator::SingleEdge(once)  => once.next(),
            GraphRayIterator::Intersection(vec) => vec.next()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::path_builder::*;

    #[test]
    fn ray_grazing_rectangle_corner() {
        let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
            .line_to(Coord2(1.0, 5.0))
            .line_to(Coord2(5.0, 5.0))
            .line_to(Coord2(5.0, 1.0))
            .line_to(Coord2(1.0, 1.0))
            .build();
        let rectangle1 = GraphPath::from_path(&rectangle1, ());

        assert!(rectangle1.ray_is_grazing(0, 0, (Coord2(2.0, 0.0), Coord2(0.0, 2.0)).coefficients()));
    }

    #[test]
    fn ray_barely_grazing_corner() {
        let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
            .line_to(Coord2(1.0, 5.0))
            .line_to(Coord2(5.0, 5.0))
            .line_to(Coord2(5.0, 1.0))
            .line_to(Coord2(1.0, 1.0))
            .build();
        let rectangle1 = GraphPath::from_path(&rectangle1, ());

        assert!(rectangle1.ray_is_grazing(0, 0, (Coord2(0.0, 1.001), Coord2(2.0, 0.999)).coefficients()));
    }

    #[test]
    fn ray_not_grazing_rectangle_corner() {
        let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
            .line_to(Coord2(1.0, 5.0))
            .line_to(Coord2(5.0, 5.0))
            .line_to(Coord2(5.0, 1.0))
            .line_to(Coord2(1.0, 1.0))
            .build();
        let rectangle1 = GraphPath::from_path(&rectangle1, ());

        assert!(!rectangle1.ray_is_grazing(0, 0, (Coord2(0.0, 0.0), Coord2(5.0, 5.0)).coefficients()));
    }

    #[test]
    fn ray_barely_not_grazing_corner() {
        let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
            .line_to(Coord2(1.0, 5.0))
            .line_to(Coord2(5.0, 5.0))
            .line_to(Coord2(5.0, 1.0))
            .line_to(Coord2(1.0, 1.0))
            .build();
        let rectangle1 = GraphPath::from_path(&rectangle1, ());

        assert!(!rectangle1.ray_is_grazing(0, 0, (Coord2(0.0, 0.9999), Coord2(2.0, 1.0001)).coefficients()));
    }
}
