use super::path::*;
use super::super::curve::*;
use super::super::intersection::*;
use super::super::super::geo::*;
use super::super::super::line::*;
use super::super::super::coordinate::*;

use std::ops::Range;
use std::fmt;
use std::mem;

const CLOSE_DISTANCE: f64 = 0.01;

///
/// Kind of a graph path edge
/// 
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GraphPathEdgeKind {
    /// An edge that hasn't been categorised yet
    Uncategorised,

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
    edge_idx: usize
}

///
/// Enum representing an edge in a graph path
/// 
#[derive(Clone, Debug)]
struct GraphPathEdge<Point> {
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
struct GraphPathPoint<Point> {
    /// The position of this point
    position: Point,

    /// The edges attached to this point
    forward_edges: Vec<GraphPathEdge<Point>>
}

impl<Point> GraphPathPoint<Point> {
    ///
    /// Creates a new graph path point
    ///
    fn new(position: Point, forward_edges: Vec<GraphPathEdge<Point>>) -> GraphPathPoint<Point> {
        GraphPathPoint { position, forward_edges }
    }
}

impl<Point: Coordinate> GraphPathEdge<Point> {
    ///
    /// Creates a new graph path edge
    /// 
    #[inline]
    fn new(kind: GraphPathEdgeKind, (cp1, cp2): (Point, Point), end_idx: usize) -> GraphPathEdge<Point> {
        GraphPathEdge {
            kind, cp1, cp2, end_idx
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
#[derive(Clone, Debug)]
pub struct GraphPath<Point> {
    /// The points in this graph and their edges. Each 'point' here consists of two control points and an end point
    points: Vec<GraphPathPoint<Point>>
}

impl<Point: Coordinate> Geo for GraphPath<Point> {
    type Point = Point;
}

impl<Point: Coordinate+Coordinate2D> GraphPath<Point> {
    ///
    /// Creates a graph path from a bezier path
    /// 
    pub fn from_path<P: BezierPath<Point=Point>>(path: &P) -> GraphPath<Point> {
        // All edges are exterior for a single path
        let mut points = vec![];

        // Push the start point (with an open path)
        let start_point = path.start_point();
        points.push(GraphPathPoint::new(start_point, vec![]));

        // We'll add edges to the previous point
        let mut last_point = 0;
        let mut next_point = 1;

        // Iterate through the points in the path
        for (cp1, cp2, end_point) in path.points() {
            // Push the points
            points.push(GraphPathPoint::new(end_point, vec![]));

            // Add an edge from the last point to the next point
            points[last_point].forward_edges.push(GraphPathEdge::new(GraphPathEdgeKind::Uncategorised, (cp1, cp2), next_point));

            // Update the last/next pooints
            last_point += 1;
            next_point += 1;
        }

        // Close the path
        if last_point > 0 {
            // Graph actually has some edges
            if start_point.distance_to(&points[last_point].position) < CLOSE_DISTANCE {
                // Remove the last point (we're replacing it with an edge back to the start)
                points.pop();
                last_point -= 1;

                // Change the edge to point back to the start
                points[last_point].forward_edges[0].end_idx = 0;
            } else {
                // Need to draw a line to the last point
                let close_vector    = points[last_point].position - start_point;
                let cp1             = close_vector * 0.33;
                let cp2             = close_vector * 0.66;

                points[last_point].forward_edges.push(GraphPathEdge::new(GraphPathEdgeKind::Uncategorised, (cp1, cp2), 0));
            }
        } else {
            // Just a start point and no edges: remove the start point as it doesn't really make sense
            points.pop();
        }

        // Create the graph path from the points
        GraphPath {
            points: points
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
    /// Returns an iterator of the edges connected to a particular point
    ///
    #[inline]
    pub fn edges<'a>(&'a self, point_num: usize) -> impl 'a+Iterator<Item=GraphEdge<'a, Point>> {
        (0..(self.points[point_num].forward_edges.len()))
            .into_iter()
            .map(move |edge_idx| GraphEdge::new(self, GraphEdgeRef { start_idx: point_num, edge_idx: edge_idx }))
    }

    ///
    /// Merges in another path
    /// 
    /// This adds the edges in the new path to this path without considering if they are internal or external 
    ///
    pub fn merge(self, merge_path: GraphPath<Point>) -> GraphPath<Point> {
        // Copy the points from this graph
        let mut new_points  = self.points;

        // Add in points from the merge path
        let offset          = new_points.len();
        new_points.extend(merge_path.points.into_iter()
            .map(|mut point| {
                // Update the offsets in the edges
                for mut edge in &mut point.forward_edges {
                    edge.end_idx += offset;
                }

                // Generate the new edge
                point
            }));

        // Combined path
        GraphPath {
            points: new_points
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
    fn join_edges_at_intersection(&mut self, edge1: (usize, usize), edge2: (usize, usize), t1: f64, t2: f64) -> Option<usize> {
        // Do nothing if the edges are the same (they're effectively already joined)
        if edge1 == edge2 { return None; }

        // Get the edge indexes
        let (edge1_idx, edge1_edge_idx) = edge1;
        let (edge2_idx, edge2_edge_idx) = edge2;

        // Create representations of the two edges
        let edge1 = Curve::from_curve(&GraphEdge::new(self, GraphEdgeRef { start_idx: edge1_idx, edge_idx: edge1_edge_idx }));
        let edge2 = Curve::from_curve(&GraphEdge::new(self, GraphEdgeRef { start_idx: edge2_idx, edge_idx: edge2_edge_idx }));

        // Create or choose a point to collide at
        // (If t1 or t2 is 0 or 1 we collide on the edge1 or edge2 points, otherwise we create a new point to collide at)
        let collision_point = if Self::t_is_zero(t1) {
            edge1_idx
        } else if Self::t_is_one(t1) {
            self.points[edge1_idx].forward_edges[edge1_edge_idx].end_idx
        } else if Self::t_is_zero(t2) {
            edge2_idx
        } else if Self::t_is_one(t2) {
            self.points[edge2_idx].forward_edges[edge1_edge_idx].end_idx
        } else {
            // Point is a mid-point of both lines

            // Work out where the mid-point is (use edge1 for this always: as this is supposed to be an intersection this shouldn't matter)
            // Note that if we use de Casteljau's algorithm here we get a subdivision for 'free' but organizing the code around it is painful
            let mid_point = edge1.point_at_pos(t1);

            // Add to this list of points
            let mid_point_idx = self.points.len();
            self.points.push(GraphPathPoint::new(mid_point, vec![]));

            // New point is the mid-point
            mid_point_idx
        };

        // Subdivide the edges
        let (edge1a, edge1b) = edge1.subdivide::<Curve<_>>(t1);
        let (edge2a, edge2b) = edge2.subdivide::<Curve<_>>(t2);

        // The new edges have the same kinds as their ancestors
        let edge1_kind      = self.points[edge1_idx].forward_edges[edge1_edge_idx].kind;
        let edge2_kind      = self.points[edge2_idx].forward_edges[edge2_edge_idx].kind;
        let edge1_end_idx   = self.points[edge1_idx].forward_edges[edge1_edge_idx].end_idx;
        let edge2_end_idx   = self.points[edge2_idx].forward_edges[edge2_edge_idx].end_idx;

        // The 'b' edges both extend from our mid-point to the existing end point (provided
        // t < 1.0)
        if !Self::t_is_one(t1) && !Self::t_is_zero(t1) {
            // If t1 is zero or one, we're not subdividing edge1
            // If zero, we're just adding the existing edge again to the collision point (so we do nothing)
            self.points[collision_point].forward_edges.push(GraphPathEdge::new(edge1_kind, edge1b.control_points(), edge1_end_idx));
        }
        if !Self::t_is_one(t2) && !Self::t_is_zero(t2) {
            // If t2 is zero or one, we're not subdividing edge2
            // If zero, we're just adding the existing edge again to the collision point (so we do nothing)
            self.points[collision_point].forward_edges.push(GraphPathEdge::new(edge2_kind, edge2b.control_points(), edge2_end_idx));
        }

        // The 'a' edges both update the initial edge, provided t is not 0
        if !Self::t_is_zero(t1) && !Self::t_is_one(t1) {
            self.points[edge1_idx].forward_edges[edge1_edge_idx].set_control_points(edge1a.control_points(), collision_point);

            // If t1 is zero, we're not subdividing edge1
            // If t1 is one this should leave the edge alone
            // If t1 is not one, then the previous step will have added the remaining part of
            // edge1 to the collision point
        }
        if !Self::t_is_zero(t2) {
            self.points[edge2_idx].forward_edges[edge2_edge_idx].set_control_points(edge2a.control_points(), collision_point);

            // If t1 is one, this should leave the edge alone
            if Self::t_is_one(t2) {
                // If t2 is one, this will have redirected the end point of t2 to the collision point: we need to move all of the edges
                let mut edge2_end_edges = vec![];
                mem::swap(&mut self.points[edge2_end_idx].forward_edges, &mut edge2_end_edges);
                self.points[collision_point].forward_edges.extend(edge2_end_edges);
            }
        }
        
        if Self::t_is_zero(t2) && collision_point != edge2_idx {
            // If t2 is zero and the collision point is not the start of edge2, then edge2 should start at the collision point instead of where it does now

            // All edges that previously went to the collision point now go to the collision point
            for point in self.points.iter_mut() {
                for edge in point.forward_edges.iter_mut() {
                    if edge.end_idx == edge2_idx {
                        edge.end_idx = collision_point;
                    }
                }
            }

            // All edges that currently come from edge2 need to be moved to the collision point
            let mut edge2_edges = vec![];
            mem::swap(&mut self.points[edge2_idx].forward_edges, &mut edge2_edges);
            self.points[collision_point].forward_edges.extend(edge2_edges);
        }

        Some(collision_point)
    }

    ///
    /// Searches two ranges of points in this object and detects collisions between them, subdividing the edges
    /// and creating branch points at the appropriate places.
    /// 
    fn detect_collisions(&mut self, collide_from: Range<usize>, collide_to: Range<usize>, accuracy: f64) {
        // Put the collide_to items in a vec, so if we subdivide any of these items, we can re-read them next time through
        let collide_to = collide_to.into_iter().collect::<Vec<_>>();

        // Vector of all of the collisions found in the graph
        let mut collisions = vec![];

        // TODO: for complicated paths, maybe some pre-processing for bounding boxes to eliminate trivial cases would be beneficial for performance

        // The points that have had collisions exactly on them (we only collide them once)
        let mut collided = vec![false; self.points.len()];

        // Iterate through the edges in the 'from' range
        for src_idx in collide_from {
            for src_edge_idx in 0..self.points[src_idx].forward_edges.len() {
                // Compare to each point in the collide_to range
                for tgt_idx in collide_to.iter() {
                    for tgt_edge_idx in 0..self.points[*tgt_idx].forward_edges.len() {
                        // Don't collide edges against themselves
                        if src_idx == *tgt_idx && src_edge_idx == tgt_edge_idx { continue; }

                        // Create edge objects for each side
                        let src_edge            = &self.points[src_idx].forward_edges[src_edge_idx];
                        let tgt_edge            = &self.points[*tgt_idx].forward_edges[tgt_edge_idx];
                        let src_curve           = GraphEdge::new(self, GraphEdgeRef { start_idx: src_idx, edge_idx: src_edge_idx });
                        let tgt_curve           = GraphEdge::new(self, GraphEdgeRef { start_idx: *tgt_idx, edge_idx: tgt_edge_idx});

                        // Quickly reject edges with non-overlapping bounding boxes
                        let src_edge_bounds     = src_curve.fast_bounding_box::<Bounds<_>>();
                        let tgt_edge_bounds     = tgt_curve.fast_bounding_box::<Bounds<_>>();
                        if !src_edge_bounds.overlaps(&tgt_edge_bounds) { continue; }

                        // Find the collisions between these two edges (these a)
                        let curve_collisions    = curve_intersects_curve_clip(&src_curve, &tgt_curve, accuracy);

                        // The are the points we need to divide the existing edges at and add branches
                        let tgt_idx = *tgt_idx;
                        for (src_t, tgt_t) in curve_collisions {
                            // A collision at t=1 is the same as a collision on t=0 on a following edge
                            // Edge doesn't actually matter for these (as the point will collide with )
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

                            // Add this as a collision
                            collisions.push(((src_idx, src_edge_idx, src_t), (tgt_idx, tgt_edge_idx, tgt_t)));
                        }
                    }
                }
            }
        }

        // Apply the divisions to the edges
        while let Some(((src_idx, src_edge, src_t), (tgt_idx, tgt_edge, tgt_t))) = collisions.pop() {
            // Join the edges
            let new_mid_point = self.join_edges_at_intersection((src_idx, src_edge), (tgt_idx, tgt_edge), src_t, tgt_t);

            // Update the remainder of the collisions if any point at the source or target edge
            if let Some(new_mid_point) = new_mid_point {
                // Usually new_mid_point is a new point, but it can be an existing point in the event the collision was at an existing point on the path

                // TODO(?): this just iterates through the collisions, not clear if this will always be fast enough
                for ((ref mut other_src_idx, ref mut other_src_edge, ref mut other_src_t), (ref mut other_tgt_idx, ref mut other_tgt_edge, ref mut other_tgt_t)) in collisions.iter_mut() {
                    // If the src edge was divided...
                    if other_src_idx == &src_idx && other_src_edge == &src_edge {
                        if *other_src_t < src_t {
                            // Before the midpoint. Edge is the same, just needs to be modified.
                            *other_src_t /= src_t;
                        } else {
                            // After the midpoint. Edge needs to be adjusted. Source edge is always the first on the midpoint
                            *other_src_t     = (*other_src_t - src_t) / (1.0-src_t);
                            *other_src_idx   = new_mid_point;
                            *other_src_edge  = 0;
                        }
                    }

                    // If the target edge was divided...
                    if other_tgt_idx == &tgt_idx && other_tgt_edge == &tgt_edge {
                        if *other_tgt_t < tgt_t {
                            // Before the midpoint. Edge is the same, just needs to be modified.
                            *other_tgt_t /= tgt_t;
                        } else {
                            // After the midpoint. Edge needs to be adjusted. Target edge is always the second on the midpoint.
                            *other_tgt_t     = (*other_tgt_t - tgt_t) / (1.0-tgt_t);
                            *other_tgt_idx   = new_mid_point;
                            *other_tgt_edge  = 1;
                        }
                    }
                }
            }
        }
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
    pub fn collide(mut self, collide_path: GraphPath<Point>, accuracy: f64) -> GraphPath<Point> {
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
    /// Finds the exterior edge (and t value) where a line first collides with this path (closest to the line
    /// start point)
    /// 
    pub fn line_collision<'a, L: Line<Point=Point>>(&'a self, ray: &L) -> Option<(GraphEdge<'a, Point>, f64)> {
        // We'll store the result after visiting all of the edges
        let mut collision_result = None;

        // Visit every edge in this graph
        for point_idx in 0..(self.points.len()) {
            for edge in self.edges(point_idx) {
                // Find out where the line collides with this edge
                let collisions = curve_intersects_line(&edge, ray);

                for (curve_t, line_t, _collide_pos) in collisions {
                    collision_result = match collision_result {
                        None                                            => { Some((edge.clone(), line_t, curve_t)) },
                        Some((last_edge, last_line_t, last_curve_t))    => {
                            if line_t < last_line_t {
                                // This match is closer to the start of the line
                                Some((edge.clone(), line_t, curve_t))
                            } else {
                                // Last match is closer to the start of the line
                                Some((last_edge, last_line_t, last_curve_t))
                            }
                        }
                    };
                }
            }
        }

        // Return the result
        collision_result.map(|(edge, _line_t, curve_t)| (edge, curve_t))
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
    /// Starting at a particular point, marks any connected edge that is not marked as exterior as interior
    ///
    fn mark_connected_edges_as_interior(&mut self, start_point: usize) {
        // Points that have been visited
        let mut visited     = vec![false; self.points.len()];

        // Stack of points waiting to be visited
        let mut to_visit    = vec![];
        to_visit.push(start_point);

        while let Some(next_point) = to_visit.pop() {
            // If we've already visited this point, mark it as visited
            if visited[next_point] { continue; }
            visited[next_point] = true;

            // Mark any uncategorised edges as interior, and visit the points they connect to
            for mut edge in self.points[next_point].forward_edges.iter_mut() {
                to_visit.push(edge.end_idx);

                if edge.kind == GraphPathEdgeKind::Uncategorised {
                    edge.kind = GraphPathEdgeKind::Interior;
                }
            }
        }
    }

    ///
    /// Given a descision function, determines which edges should be made exterior. The start edge is always made external.
    /// Any edges connected to the start edge that are not picked by the picking function are marked as interior.
    ///
    /// This can be used to implement path arithmetic algorithms by deciding which edges from the shared path should
    /// become the exterior edges of a new path.
    ///
    /// The picking function is supplied a list of possible edges and should pick the edge that represents the following
    /// exterior edge.
    ///
    pub fn classify_exterior_edges<PickEdgeFn>(&mut self, start_edge: GraphEdgeRef, pick_external_edge: PickEdgeFn)
    where PickEdgeFn: Fn(&Self, &Vec<GraphEdge<'_, Point>>) -> usize {
        let mut current_point_idx   = start_edge.start_idx;
        let mut current_edge_idx    = start_edge.edge_idx;

        loop {
            // If we've arrived back at an exterior edge, we've finished marking edges as exterior
            if self.points[current_point_idx].forward_edges[current_edge_idx].kind == GraphPathEdgeKind::Exterior {
                break;
            }
            
            // Mark the current edge as exterior
            self.points[current_point_idx].forward_edges[current_edge_idx].kind = GraphPathEdgeKind::Exterior;

            // Update the current point to be the end of this edge
            current_point_idx = self.points[current_point_idx].forward_edges[current_edge_idx].end_idx;

            // Fetch the next external edge using the decision function (pick_external_edge)
            let next_edge = {
                // Gather the edges for the current point
                // TODO: we need to have the 'reverse' edges as well here
                let edges = (0..(self.points[current_point_idx].forward_edges.len()))
                    .filter(|edge_idx| self.points[current_point_idx].forward_edges[*edge_idx].kind == GraphPathEdgeKind::Uncategorised)
                    .map(|edge_idx| GraphEdge::new(self, GraphEdgeRef { start_idx: current_point_idx, edge_idx: edge_idx }))
                    .collect();

                // Pass to the selection function to pick the next edge we go to
                pick_external_edge(self, &edges)
            };

            // Set the current edge
            current_edge_idx = next_edge;
        }

        // Go around the loop again and mark any edges still uncategorized as interior
        self.mark_connected_edges_as_interior(current_point_idx);
    }
}

///
/// Represents an edge in a graph path
/// 
#[derive(Clone)]
pub struct GraphEdge<'a, Point: 'a> {
    /// The graph that this point is for
    graph: &'a GraphPath<Point>,

    /// A reference to the edge this point is for
    edge: GraphEdgeRef
}

impl<'a, Point: 'a> GraphEdge<'a, Point> {
    ///
    /// Creates a new graph edge (with an edge kind of 'exterior')
    /// 
    #[inline]
    fn new(graph: &'a GraphPath<Point>, edge: GraphEdgeRef) -> GraphEdge<'a, Point> {
        GraphEdge {
            graph:  graph,
            edge:   edge
        }
    }

    fn edge<'b>(&'b self) -> &'b GraphPathEdge<Point> {
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
        self.edge.start_idx
    }

    ///
    /// Returns the index of the end point of this edge
    /// 
    #[inline]
    pub fn end_point_index(&self) -> usize {
        self.edge().end_idx
    }
}

impl<'a, Point: 'a+Coordinate> Geo for GraphEdge<'a, Point> {
    type Point = Point;
}

impl<'a, Point: 'a+Coordinate> BezierCurve for GraphEdge<'a, Point> {
    ///
    /// The start point of this curve
    /// 
    #[inline]
    fn start_point(&self) -> Self::Point {
        self.graph.points[self.edge.start_idx].position.clone()
    }

    ///
    /// The end point of this curve
    /// 
    #[inline]
    fn end_point(&self) -> Self::Point {
        self.graph.points[self.edge().end_idx].position.clone()
    }

    ///
    /// The control points in this curve
    /// 
    #[inline]
    fn control_points(&self) -> (Self::Point, Self::Point) {
        let edge = self.edge();
        (edge.cp1.clone(), edge.cp2.clone())
    }
}

///
/// A GraphEdgeRef can be created from a GraphEdge in order to release the borrow
///
impl<'a, Point: 'a+Coordinate> From<GraphEdge<'a, Point>> for GraphEdgeRef {
    fn from(edge: GraphEdge<'a, Point>) -> GraphEdgeRef {
        edge.edge
    }
}

impl<'a, Point: fmt::Debug> fmt::Debug for GraphEdge<'a, Point> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} -> {:?} ({:?} -> {:?} ({:?}, {:?}))", self.edge.start_idx, self.edge().end_idx, self.graph.points[self.edge.start_idx].position, self.graph.points[self.edge().end_idx].position, self.edge().cp1, self.edge().cp2)
    }
}