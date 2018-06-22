use super::path::*;
use super::bounds::*;
use super::super::curve::*;
use super::super::normal::*;
use super::super::intersection::*;
use super::super::super::coordinate::*;

///
/// Returns true if a particular point is within a bezier path
/// 
pub fn path_contains_point<P: BezierPath>(path: &P, point: &P::Point)
where P::Point: Coordinate2D {
}
