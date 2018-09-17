use super::fat_line::*;
use super::super::super::bezier::*;

///
/// Determines the points at which two curves intersect using the Bezier clipping
/// algorihtm
/// 
pub fn curve_intersects_curve_clip<'a, C: BezierCurve>(curve1: &'a C, curve2: &'a C)
where C::Point: 'a+Coordinate2D {
    // Create the fat line from the first curve
    let fat_line = FatLine::from_curve(curve1);

    // Clip the second curve to the line
    let curve2_clip = fat_line.clip::<_, Curve<C::Point>>(curve2);
}
