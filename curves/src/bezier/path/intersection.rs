use super::path::*;
use super::to_curves::*;
use super::super::curve::*;
use super::super::intersection::*;
use super::super::super::line::*;
use super::super::super::coordinate::*;

///
/// Determines the intersections of a path and a line
/// 
/// Intersections are returned as the path section index and the 't' parameter along that curve
/// 
pub fn path_intersects_line<'a, Path: BezierPath, L: Line<Point=Path::Point>>(path: &'a Path, line: &'a L) -> impl 'a+Iterator<Item=(usize, f64)> 
where Path::Point: 'a+Coordinate2D {
    path_to_curves::<_, Curve<_>>(path)
        .enumerate()
        .flat_map(move |(section_id, curve)| curve_intersects_line(&curve, line).into_iter().map(move |t| (section_id, t)))
}
