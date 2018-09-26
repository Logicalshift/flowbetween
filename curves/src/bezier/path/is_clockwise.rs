use super::path::*;
use super::super::super::coordinate::*;

use itertools::*;

///
/// Determines if a set of points are in a clockwise ordering (assuming that a positive y value indicates an upwards direction)
///
pub fn points_are_clockwise<Point: Coordinate+Coordinate2D, PointIter: Iterator<Item=Point>>(mut points: PointIter) -> bool {
    // Technique suggested in https://stackoverflow.com/questions/1165647/how-to-determine-if-a-list-of-polygon-points-are-in-clockwise-order
    let mut total = 0.0;

    // The first point needs to be repeated at the end of the sequence
    let first_point = points.next();
    if let Some(first_point) = first_point {
        let points = vec![first_point.clone()].into_iter().chain(points).chain(vec![first_point].into_iter());

        // Sum over the edges to determine if the points are clockwise
        for (start, end) in points.tuple_windows() {
            total += (end.x()-start.x()) * (end.y()+start.y());
        }
    }

    total >= 0.0
}

///
/// Trait implemented by paths that can determine if their points are in a clockwise ordering or not
///
pub trait PathWithIsClockwise {
    ///
    /// Determines if this path is ordered in a clockwise direction
    ///
    fn is_clockwise(&self) -> bool;
}

impl<P: BezierPath> PathWithIsClockwise for P where P::Point: Coordinate+Coordinate2D {
    #[inline]
    fn is_clockwise(&self) -> bool {
        points_are_clockwise(self.points().map(|(_cp1, _cp2, p)| p))
    }
}
