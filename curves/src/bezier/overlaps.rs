use super::curve::*;
use super::section::*;
use super::super::line::*;
use super::super::consts::*;
use super::super::coordinate::*;

///
/// If `curve2` overlaps `curve1`, returns two sets of `t` values (those for `curve1` and those for `curve2`)
///
pub fn overlapping_region<P: Coordinate+Coordinate2D, C1: BezierCurve<Point=P>, C2: BezierCurve<Point=P>>(curve1: &C1, curve2: &C2) -> Option<((f64, f64), (f64, f64))> {
    let mut c2_t1 = 0.0;
    let mut c2_t2 = 1.0;

    // The start and end points of curve1 should be on curve2
    let c2_start    = curve2.start_point();
    let c2_end      = curve2.end_point();

    let c1_t1 = if let Some(t) = curve1.t_for_point(&c2_start) {
        // Start point is on the curve
        t
    } else if let Some(t) = curve2.t_for_point(&curve1.start_point()) {
        // curve1 starts on a point of curve2
        c2_t1 = t;
        0.0
    } else {
        // Neither point is on the curve
        return None;
    };

    let c1_t2 = if let Some(t) = curve1.t_for_point(&c2_end) {
        // End point is on the curve
        t
    } else if let Some(t) = curve2.t_for_point(&curve1.end_point()) {
        // curve1 ends on a point of curve2
        c2_t2 = t;
        1.0
    } else {
        // End point is not on the curve
        return None;
    };

    // If curve1 and curve2 are collinear - two overlapping lines - we've already got the results (and the control points will differ anyway)
    #[inline]
    fn is_collinear<P: Coordinate2D>(p: &P, &(a, b, c): &(f64, f64, f64)) -> bool {
        (a*p.x() + b*p.y() + c).abs() < SMALL_DISTANCE
    }

    let coeff               = (curve1.start_point(), curve1.end_point()).coefficients();
    let (c1_cp1, c1_cp2)    = curve1.control_points();

    if is_collinear(&c1_cp1, &coeff) && is_collinear(&c1_cp2, &coeff)
        && is_collinear(&curve2.start_point(), &coeff) && is_collinear(&curve2.end_point(), &coeff) {
        let (c2_cp1, c2_cp2) = curve2.control_points();

        if is_collinear(&c2_cp1, &coeff) && is_collinear(&c2_cp2, &coeff) {
            return Some(((c1_t1, c1_t2), (c2_t1, c2_t2)));
        }
    }

    // Start and end points match at t1, t2
    #[inline]
    fn close_enough<P: Coordinate>(p1: &P, p2: &P) -> bool {
        let offset = *p1 - *p2;
        offset.dot(&offset) < (SMALL_DISTANCE * SMALL_DISTANCE)
    }

    // Get the control points for the two curves
    #[inline]
    fn control_points<P: Coordinate+Coordinate2D, C: BezierCurve<Point=P>>(curve: &C, t1: f64, t2: f64) -> (P, P) {
        if t2 < t1 {
            let (cp2, cp1) = curve.section(t2, t1).control_points();
            (cp1, cp2)
        } else {
            curve.section(t1, t2).control_points()
        }
    }

    let (c2_cp1, c2_cp2) = if c2_t1 != 0.0 || c2_t2 != 1.0 {
        control_points(curve2, c2_t1, c2_t2)
    } else {
        curve2.control_points()
    };

    let (c1_cp1, c1_cp2) = control_points(curve1, c1_t1, c1_t2);

    // If they're about the same, we've found an overlapping region
    if close_enough(&c1_cp1, &c2_cp1) && close_enough(&c1_cp2, &c2_cp2) {
        Some(((c1_t1, c1_t2), (c2_t1, c2_t2)))
    } else {
        None
    }
}
