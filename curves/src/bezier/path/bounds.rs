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
        .map(|(min, max)| (P::Point::from_smallest_components(min, max), P::Point::from_biggest_components(min, max)))
        .fold1(|(min1, max1), (min2, max2)| (P::Point::from_smallest_components(min1, min2), P::Point::from_biggest_components(max1, max2)))
        .map(|b| Bounds::from_min_max(b.0, b.1))
        .unwrap_or_else(|| Bounds::from_min_max(P::Point::origin(), P::Point::origin()))
}
