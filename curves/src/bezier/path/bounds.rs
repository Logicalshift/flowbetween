use super::path::*;
use super::to_curves::*;
use super::super::curve::*;
use super::super::super::geo::*;
use super::super::super::coordinate::*;

use itertools::*;

///
/// Finds the bounds of a path
/// 
pub fn path_bounding_box<P: BezierPath, Bounds: BoundingBox<Point=P::Point>>(path: &P) -> Bounds {
    path_to_curves(path)
        .map(|curve: Curve<P::Point>| curve.bounding_box())
        .fold1(|first: Bounds, second| first.union_bounds(second))
        .unwrap_or_else(|| Bounds::from_min_max(P::Point::origin(), P::Point::origin()))
}
