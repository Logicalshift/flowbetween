use super::curve::*;
use super::normal::*;
use super::section::*;
use super::super::line::*;
use super::super::coordinate::*;

use std::cmp::Ordering;

///
/// Returns true if the specified bezier curve is 'safe'
/// 
/// A safe curve has both control points on the same side of the base line and the point at t=0.5
/// roughly in the center of the polygon formed by the points of the curve
///
fn is_safe_curve<Curve: BezierCurve>(curve: &Curve) -> bool
where Curve::Point: Coordinate2D {
    // Get the points of the curve
    let start_point = curve.start_point();
    let end_point   = curve.end_point();
    let (cp1, cp2)  = curve.control_points();

    // Determine if the two control points are on the same side
    let (a, b, c)   = line_coefficients_2d_unnormalized(&(start_point, end_point));
    let side_cp1    = (a*cp1.x() + b*cp1.y() + c).signum();
    let side_cp2    = (a*cp2.x() + b*cp2.y() + c).signum();

    debug_assert!(!side_cp1.is_nan());
    debug_assert!(!side_cp2.is_nan());

    if side_cp1 != side_cp2 {
        // Control points are on different sides
        false
    } else {
        // Mid point of the polygon is the average of all of the points
        let polygon_mid_point   = (start_point + cp1 + cp2 + end_point) * 0.25;

        // Maximum distance from the mid point to consider the curve 'safe'
        let max_mid_distance    = start_point.distance_to(&end_point) * 0.1;
        let max_mid_distance    = max_mid_distance.max(5.0);

        // Is safe if the point at t = 0.5 is within this distance of the midpoint
        let curve_mid_point     = curve.point_at_pos(0.5);
        curve_mid_point.is_near_to(&polygon_mid_point, max_mid_distance)
    }
}

///
/// Computes a series of curves that approximate an offset curve from the specified origin curve.
/// 
/// Based on the algorithm described in https://pomax.github.io/bezierinfo/#offsetting
///
pub fn offset<Curve: BezierCurveFactory+NormalCurve>(curve: &Curve, initial_offset: f64, final_offset: f64) -> Vec<Curve>
where Curve::Point: Normalize+Coordinate2D {
    // Cut the curve up into 'safe' sections
    let mut sections = vec![curve.section(0.0, 1.0)];
    
    if !is_safe_curve(curve) {
        // Start by splitting at the extreme points
        let mut extremes = curve.find_extremities();

        extremes.retain(|t| !t.is_nan() && !t.is_infinite() && t > &0.0 && t < &1.0);
        extremes.sort_by(|t1, t2| t1.partial_cmp(t2).unwrap_or(Ordering::Equal));

        // Split up the curve into subsections at the extreme points
        sections.clear();
        let mut last_t = 0.0;

        for t in extremes {
            sections.push(curve.section(last_t, t));
            last_t = t;
        }

        if last_t < 1.0 {
            sections.push(curve.section(last_t, 1.0));
        }

        // Split 'unsafe' sections into two until all sections are safe
        loop {
            let mut all_safe    = true;
            debug_assert!(sections.len() < 50);

            // Check all of the sections
            let mut section_idx = 0;
            while section_idx < sections.len() {
                // Split this section if it's not safe
                if !is_safe_curve(&sections[section_idx]) {
                    all_safe = false;

                    let left    = sections[section_idx].subsection(0.0, 0.5);
                    let right   = sections[section_idx].subsection(0.5, 1.0);

                    sections[section_idx] = left;
                    sections.insert(section_idx+1, right);

                    section_idx += 1;
                }

                section_idx += 1;
            }

            // Stop once all sections are safe
            if all_safe { break; }
        }
    }

    // Offset the set of curves that we retrieved
    let offset_distance = final_offset-initial_offset;

    sections.into_iter()
        .map(|section| {
            // Compute the offsets for this section (TODO: use the curve length, not the t values)
            let (t1, t2)            = section.original_curve_t_values();
            let (offset1, offset2)  = (t1*offset_distance+initial_offset, t2*offset_distance+initial_offset);

            simple_offset(&section, offset1, offset2).0
        })
        .collect()
}

///
/// Computes the offset error between a curve and a proposed offset curve at a given t value
/// 
#[inline]
fn offset_error<Curve: NormalCurve>(original_curve: &Curve, offset_curve: &Curve, t: f64, initial_offset: f64, final_offset: f64) -> Curve::Point {
    // Work out how much we need to offset the mid-point
    let midpoint_offset     = (final_offset - initial_offset) * (original_curve.estimate_length(t)/original_curve.estimate_length(1.0)) + initial_offset;
    let midpoint_normal     = original_curve.normal_at_pos(t).to_unit_vector();
    let original_midpoint   = original_curve.point_at_pos(t);
    let new_midpoint        = offset_curve.point_at_pos(t);
    let target_pos          = original_midpoint + midpoint_normal*midpoint_offset;
    let offset_error        = target_pos - new_midpoint;

    offset_error
}

///
/// Offsets the endpoints and mid-point of a curve by the specified amounts without subdividing
/// 
/// This won't produce an accurate offset if the curve doubles back on itself. The return value is the curve and the error
/// 
fn simple_offset<P: Coordinate, CurveIn: NormalCurve+BezierCurve<Point=P>, CurveOut: BezierCurveFactory<Point=P>>(curve: &CurveIn, initial_offset: f64, final_offset: f64) -> (CurveOut, f64) 
where P: Normalize {
    // Fetch the original points
    let start       = curve.start_point();
    let end         = curve.end_point();
    let (cp1, cp2)  = curve.control_points();

    // The start and end CPs define the curve tangents at the start and end
    let normal_start    = P::to_normal(&start, &(cp1-start));
    let normal_end      = P::to_normal(&end, &(end-cp2));
    let normal_start    = P::from_components(&normal_start).to_unit_vector();
    let normal_end      = P::from_components(&normal_end).to_unit_vector();

    // Offset start & end by the specified amounts to create the first approximation of a curve
    // TODO: scale rather than just move for better accuracy
    let new_start   = start + (normal_start * initial_offset);
    let new_cp1     = cp1 + (normal_start * initial_offset);
    let new_cp2     = cp2 + (normal_end * final_offset);
    let new_end     = end + (normal_end * final_offset);

    let offset_curve = CurveOut::from_points(new_start, new_end, new_cp1, new_cp2);

    let error = 0.0;
    /*
    // Tweak the curve at some sample points to improve the accuracy of our guess
    for sample_t in [0.25, 0.75].into_iter() {
        let sample_t = *sample_t;

        // Work out th error at this point
        let move_offset = offset_error(curve, &offset_curve, sample_t, initial_offset, final_offset);

        // Adjust the curve by the offset
        offset_curve = move_point(&offset_curve, sample_t, move_offset);
    }

    // Use the offset at the curve's midway point as the error
    let error_offset    = offset_error(curve, &offset_curve, 0.5, initial_offset, final_offset);
    let error           = Curve::Point::origin().distance_to(&error_offset);
    */

    (offset_curve, error)
}
