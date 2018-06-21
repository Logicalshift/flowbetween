use super::curve::*;
use super::super::line::*;
use super::super::coordinate::*;

///
/// Find the t values where a curve intersects a line
/// 
pub fn curve_intersects_line<C: BezierCurve, L: Line<Point=C::Point>>(curve: &C, line: &L) -> Vec<f64>
where C::Point: Coordinate2D {
    curve.search_with_bounds(0.01, |min, max| line_clip_to_bounds(line, &(min, max)).is_some())
}
