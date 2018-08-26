use super::path::*;
use super::super::curve::*;
use super::super::super::geo::*;
use super::super::super::coordinate::*;

const CLOSE_DISTANCE: f64 = 0.01;

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
                points[1].1 = close_vector * 0.66;
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
}

///
/// Represents an edge in a graph path
/// 
#[derive(Clone)]
pub struct GraphEdge<'a, Point: 'a> {
    /// The graph that this point is for
    graph: &'a GraphPath<Point>,

    /// The initial point of this edge
    start_point: usize,

    /// The end point of this edge
    end_point: usize
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
