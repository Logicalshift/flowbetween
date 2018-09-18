use super::fat_line::*;
use super::super::super::bezier::*;

///
/// Determines the length of a curve's hull as a sum of squares
/// 
fn curve_hull_length_sq<C: BezierCurve>(curve: &C) -> f64 {
    let start       = curve.start_point();
    let end         = curve.end_point();
    let (cp1, cp2)  = curve.control_points();

    let offset1 = cp1-start;
    let offset2 = cp2-cp1;
    let offset3 = cp2-end;

    offset1.dot(&offset1) + offset2.dot(&offset2) + offset3.dot(&offset3)
}

///
/// Determines the points at which two curves intersect using the Bezier clipping algorithm
/// 
fn curve_intersects_curve_clip_inner<'a, C: BezierCurveFactory>(curve1: C, curve2: C, accuracy_squared: f64) -> Vec<(f64, f64)>
where C::Point: 'a+Coordinate2D {
    // Create the fat line from the first curve
    let fat_line = FatLine::from_curve(&curve1);

    // Clip the second curve to the line
    let curve2_clip = fat_line.clip::<_, Curve<C::Point>>(&curve2);

    unimplemented!()
}

///
/// Determines the points at which two curves intersect using the Bezier clipping
/// algorihtm
/// 
pub fn curve_intersects_curve_clip<'a, C: BezierCurve>(curve1: &'a C, curve2: &'a C, accuracy: f64) -> Vec<(f64, f64)>
where C::Point: 'a+Coordinate2D {
    // Convert to the standard curve type
    let mut curve1 = Curve::from_curve(curve1);
    let mut curve2 = Curve::from_curve(curve2);

    // Perform the clipping algorithm on these curves
    curve_intersects_curve_clip_inner(curve1, curve2, accuracy*accuracy)
}

