use super::path::*;
use super::super::super::geo::*;
use super::super::super::coordinate::*;

///
/// Enum representing an edge in a graph path. There are three points associated with an edge: two control points
/// and an end point
/// 
pub enum GraphPathEdge {
    /// An exterior edge
    Exterior(usize, usize, usize),

    /// An interior edge
    Interior(usize, usize, usize)
}

///
/// A graph path is a path where each point can have more than one connected edge. Edges are categorized
/// into interior and exterior edges depending on if they are on the outside or the inside of the combined
/// shape.
/// 
pub struct GraphPath<Point> {
    /// The points in this graph and their edges
    points: Vec<(Point, Vec<GraphPathEdge>)>
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

        // Push the start point
        let start_point = path.start_point();
        points.push((start_point, vec![]));

        // We'll add edges to the previous point
        let mut last_point = 0;
        let mut next_point = 1;

        // Iterate through the points in the path
        for (cp1, cp2, end_point) in path.points() {
            // Push the points
            points.push((cp1, vec![]));
            points.push((cp2, vec![]));
            points.push((end_point, vec![]));

            // Indexes for the points
            let cp1         = next_point;
            let cp2         = next_point+1;
            let end_point   = next_point+2;

            // Add an edge from the last point to the next point
            points[last_point].1.push(GraphPathEdge::Exterior(cp1, cp2, end_point));

            // Update the last/next pooints
            last_point += 3;
            next_point += 3;
        }

        // Create the graph path from the points
        GraphPath {
            points: points
        }
    }
}