use super::path::*;
use super::super::curve::*;
use super::super::intersection::*;
use super::super::super::geo::*;
use super::super::super::coordinate::*;

use std::ops::Range;

const CLOSE_DISTANCE: f64 = 0.01;

///
/// Kind of a graph path edge
/// 
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GraphPathEdgeKind {
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

impl<Point: Coordinate> GraphPathEdge<Point> {
    ///
    /// Creates a new graph path edge
    /// 
    fn new(kind: GraphPathEdgeKind, cp1: Point, cp2: Point, end_idx: usize) -> GraphPathEdge<Point> {
        GraphPathEdge {
            kind, cp1, cp2, end_idx
        }
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
    points: Vec<(Point, Vec<GraphPathEdge<Point>>)>
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
        points.push((start_point, vec![]));

        // We'll add edges to the previous point
        let mut last_point = 0;
        let mut next_point = 1;

        // Iterate through the points in the path
        for (cp1, cp2, end_point) in path.points() {
            // Push the points
            points.push((end_point, vec![]));

            // Add an edge from the last point to the next point
            points[last_point].1.push(GraphPathEdge::new(GraphPathEdgeKind::Exterior, cp1, cp2, next_point));

            // Update the last/next pooints
            last_point += 1;
            next_point += 1;
        }

        // Close the path
        if last_point > 0 {
            // Graph actually has some edges
            if start_point.distance_to(&points[last_point].0) < CLOSE_DISTANCE {
                // Remove the last point (we're replacing it with an edge back to the start)
                points.pop();
                last_point -= 1;

                // Change the edge to point back to the start
                points[last_point].1[0].end_idx = 0;
            } else {
                // Need to draw a line to the last point
                let close_vector    = points[last_point].0 - start_point;
                let cp1             = close_vector * 0.33;
                let cp2             = close_vector * 0.66;

                points[last_point].1.push(GraphPathEdge::new(GraphPathEdgeKind::Exterior, cp1, cp2, 0));
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
        self.points[point_num].1
            .iter()
            .map(move |edge| GraphEdge::new(self, point_num, edge))
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
            .map(|(point, mut edges)| {
                // Update the offsets in the edges
                for mut edge in &mut edges {
                    edge.end_idx += offset;
                }

                // Generate the new edge
                (point, edges)
            }));

        // Combined path
        GraphPath {
            points: new_points
        }
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

        // Iterate through the edges in the 'from' range
        for src_idx in collide_from {
            for src_edge in 0..self.points[src_idx].1.len() {
                // Compare to each point in the collide_to range
                for tgt_idx in collide_to.iter() {
                    for tgt_edge in 0..self.points[*tgt_idx].1.len() {
                        // Don't collide edges against themselves
                        if src_idx == *tgt_idx && src_edge == tgt_edge { continue; }

                        // Create edge objects for each side
                        let src_edge            = &self.points[src_idx].1[src_edge];
                        let tgt_edge            = &self.points[*tgt_idx].1[tgt_edge];
                        let src_curve           = GraphEdge::new(self, src_idx, src_edge);
                        let tgt_curve           = GraphEdge::new(self, *tgt_idx, tgt_edge);

                        // Quickly reject edges with non-overlapping bounding boxes
                        let src_edge_bounds     = src_curve.fast_bounding_box::<Bounds<_>>();
                        let tgt_edge_bounds     = tgt_curve.fast_bounding_box::<Bounds<_>>();
                        if !src_edge_bounds.overlaps(&tgt_edge_bounds) { continue; }

                        // Find the collisions between these two edges (these a)
                        let curve_collisions    = curve_intersects_curve(&src_curve, &tgt_curve, accuracy);

                        // The are the points we need to divide the existing edges at and add branches
                        for (src_t, tgt_t) in curve_collisions {
                            collisions.push(((src_idx, src_edge, src_t), (tgt_idx, tgt_edge, tgt_t)));
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
}

///
/// Represents an edge in a graph path
/// 
#[derive(Clone)]
pub struct GraphEdge<'a, Point: 'a> {
    /// The graph that this point is for
    graph: &'a GraphPath<Point>,

    /// The point where the edge starts
    start_idx: usize,

    /// The edge in the graph that this represents
    edge: &'a GraphPathEdge<Point>
}

impl<'a, Point: 'a> GraphEdge<'a, Point> {
    ///
    /// Creates a new graph edge (with an edge kind of 'exterior')
    /// 
    #[inline]
    fn new(graph: &'a GraphPath<Point>, start_idx: usize, edge: &'a GraphPathEdge<Point>) -> GraphEdge<'a, Point> {
        GraphEdge {
            graph:          graph,
            start_idx:      start_idx,
            edge:           edge
        }
    }

    ///
    /// Returns if this is an interior or an exterior edge in the path
    /// 
    pub fn kind(&self) -> GraphPathEdgeKind {
        self.edge.kind
    }

    ///
    /// Returns the index of the start point of this edge
    /// 
    #[inline]
    pub fn start_point_index(&self) -> usize {
        self.start_idx
    }

    ///
    /// Returns the index of the end point of this edge
    /// 
    #[inline]
    pub fn end_point_index(&self) -> usize {
        self.edge.end_idx
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
        self.graph.points[self.start_idx].0.clone()
    }

    ///
    /// The end point of this curve
    /// 
    #[inline]
    fn end_point(&self) -> Self::Point {
        self.graph.points[self.edge.end_idx].0.clone()
    }

    ///
    /// The control points in this curve
    /// 
    #[inline]
    fn control_points(&self) -> (Self::Point, Self::Point) {
        (self.edge.cp1.clone(), self.edge.cp2.clone())
    }
}
