use super::path::*;
use super::super::curve::*;
use super::super::super::geo::*;
use super::super::super::coordinate::*;

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
#[derive(Copy, Clone, Debug)]
pub enum GraphPathEdge {
    /// An exterior edge
    Exterior(usize),

    /// An interior edge
    Interior(usize)
}

impl GraphPathEdge {
    ///
    /// Converts this edge into a kind and a edge number
    /// 
    #[inline]
    pub fn to_kind(&self) -> (GraphPathEdgeKind, usize) {
        match self {
            GraphPathEdge::Exterior(point_index) => (GraphPathEdgeKind::Exterior, *point_index),
            GraphPathEdge::Interior(point_index) => (GraphPathEdgeKind::Interior, *point_index)
        }
    }

    ///
    /// Sets the target point index for this edge
    /// 
    #[inline]
    pub fn set_target(&mut self, new_target: usize) {
        match self {
            GraphPathEdge::Exterior(ref mut point_index) => *point_index = new_target,
            GraphPathEdge::Interior(ref mut point_index) => *point_index = new_target
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
    points: Vec<(Point, Point, Point, Vec<GraphPathEdge>)>
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
        points.push((Point::origin(), Point::origin(), start_point, vec![]));

        // We'll add edges to the previous point
        let mut last_point = 0;
        let mut next_point = 1;

        // Iterate through the points in the path
        for (cp1, cp2, end_point) in path.points() {
            // Push the points
            points.push((cp1, cp2, end_point, vec![]));

            // Add an edge from the last point to the next point
            points[last_point].3.push(GraphPathEdge::Exterior(next_point));

            // Update the last/next pooints
            last_point += 1;
            next_point += 1;
        }

        // Close the path
        if last_point > 0 {
            // Graph actually has some edges
            if start_point.distance_to(&points[last_point].2) < CLOSE_DISTANCE {
                // Start point the same as the last point. Change initial control points
                points[0].0 = points[last_point].0.clone();
                points[0].1 = points[last_point].1.clone();

                // Remove the last point (we're replacing it with an edge back to the start)
                points.pop();
                last_point -= 1;
            } else {
                // Need to draw a line to the last point
                let close_vector = points[last_point].2 - start_point;
                points[0].0 = close_vector * 0.33;
                points[0].1 = close_vector * 0.66;
            }

            // Add an edge from the start point to the end point
            points[last_point].3.push(GraphPathEdge::Exterior(0));
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
        self.points[point_num].3
            .iter()
            .map(move |edge| {
                let (kind, end_point) = edge.to_kind();
                GraphEdge {
                    kind:           kind,
                    graph:          self,
                    start_point:    point_num,
                    end_point:      end_point
                }
            })
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
            .map(|(cp1, cp2, p, mut edges)| {
                // Update the offsets in the edges
                for mut edge in &mut edges {
                    let (_, index) = edge.to_kind();
                    edge.set_target(index + offset);
                }

                // Generate the new edge
                (cp1, cp2, p, edges)
            }));

        // Combined path
        GraphPath {
            points: new_points
        }
    }
}

///
/// Represents an edge in a graph path
/// 
#[derive(Clone)]
pub struct GraphEdge<'a, Point: 'a> {
    /// The kind of edge that this represents
    kind: GraphPathEdgeKind,

    /// The graph that this point is for
    graph: &'a GraphPath<Point>,

    /// The initial point of this edge
    start_point: usize,

    /// The end point of this edge
    end_point: usize
}

impl<'a, Point: 'a> GraphEdge<'a, Point> {
    ///
    /// Returns if this is an interior or an exterior edge in the path
    /// 
    pub fn kind(&self) -> GraphPathEdgeKind {
        self.kind
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
        self.graph.points[self.start_point].2.clone()
    }

    ///
    /// The end point of this curve
    /// 
    #[inline]
    fn end_point(&self) -> Self::Point {
        self.graph.points[self.end_point].2.clone()
    }

    ///
    /// The control points in this curve
    /// 
    #[inline]
    fn control_points(&self) -> (Self::Point, Self::Point) {
        (self.graph.points[self.end_point].0.clone(), self.graph.points[self.end_point].1.clone())
    }
}
