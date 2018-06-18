use super::super::super::coordinate::*;

///
/// Trait representing a path made out of bezier sections
/// 
pub trait BezierPath {
    /// The type of a point in this path
    type Point: Coordinate;

    /// Type of an iterator over the points in this curve. This tuple contains the points ordered as a hull: ie, two control points followed by a point on the curve
    type PointIter: Iterator<Item=(Self::Point, Self::Point, Self::Point)>;

    ///
    /// Retrieves the initial point of this path
    /// 
    fn start_point(&self) -> Self::Point;

    ///
    /// Retrieves an iterator over the points in this path
    /// 
    fn points(&self) -> Self::PointIter;

    ///
    /// Creates a new instance of this path from a set of points
    /// 
    fn from_points<FromIter: IntoIterator<Item=(Self::Point, Self::Point, Self::Point)>>(start_point: Self::Point, points: FromIter) -> Self;
}
