use super::line::*;
use super::super::coordinate::*;

///
/// Returns the point at which two lines intersect (if they intersect)
/// 
/// Only the 2-dimensional form is supported at the moment (lines are much less likely to intersect
/// in higher dimensions)
/// 
pub fn line_intersects_line<L: Line>(line1: &L, line2: &L) -> Option<L::Point> 
where L::Point: Coordinate2D {
    let line1_points = line1.points();
    let line2_points = line2.points();

    let ((x1, y1), (x2, y2)) = (line1_points.0.coords(), line1_points.1.coords());
    let ((x3, y3), (x4, y4)) = (line2_points.0.coords(), line2_points.1.coords());

    let ua = ((x4-x3)*(y1-y3) - (y4-y3)*(x1-x3)) / ((y4-y3)*(x2-x1) - (x4-x3)*(y2-y1));
    let ub = ((x2-x1)*(y1-y3) - (y2-y1)*(x1-x3)) / ((y4-y3)*(x2-x1) - (x4-x3)*(y2-y1));

    if ua >= 0.0 && ua <= 1.0 && ub >= 0.0 && ub <= 1.0 {
        Some(L::Point::from_components(&[
            x1+(ua*(x2-x1)), 
            y1+(ua*(y2-y1))
        ]))
    } else {
        None
    }
}
