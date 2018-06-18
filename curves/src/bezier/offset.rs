use super::deform::*;
use super::normal::*;
use super::super::coordinate::*;

///
/// Computes a series of curves that approximate an offset curve from the specified origin curve
/// 
pub fn offset<Curve: NormalCurve>(curve: &Curve, initial_offset: f64, final_offset: f64) -> Vec<Curve> 
where Curve::Point: Normalize<Point=Curve::Point> {
    // Pass through the curve if it's 0-length
    let start       = curve.start_point();
    let end         = curve.end_point();
    let (cp1, cp2)  = curve.control_points();

    if start.distance_to(&cp1) < 0.00001 && cp1.distance_to(&cp2) < 0.00001 && cp2.distance_to(&end) < 0.00001 {
        return vec![curve.clone()];
    }

    // Split the curve at its extremities to generate a set of simpler curves
    let split_points    = curve.find_extremities();
    let curves          = split_offsets(curve, initial_offset, final_offset, &split_points);

    // Offset the curves
    let mut offset_curves   = vec![];
    let mut previous_offset = initial_offset;

    for (curve, next_offset) in curves {
        // Offset this curve
        let (offset_curve, _error) = simple_offset(curve, previous_offset, next_offset);
        offset_curves.push(offset_curve);

        // This is the initial offset of the next curve
        previous_offset = next_offset;
    }

    // TODO: check the offset curve against an error bound and subdivide further if it doesn't make it
    // TODO: we sometimes generate NaN curves (though not very often)
    // This is the final result
    offset_curves
}

///
/// Splits a curve at a given set of ordered offsets, returning a list of curves and
/// their final offsets
/// 
fn split_offsets<Curve: NormalCurve>(curve: &Curve, initial_offset: f64, final_offset: f64, split_points: &[f64]) -> Vec<(Curve, f64)> {
    let mut curves_and_offsets  = vec![];
    let mut remaining           = curve.clone();
    let mut remaining_t         = 0.0;
    
    let overall_length          = curve.estimate_length(1.0);

    for point in split_points {
        // Don't subdivide at point 0 (it doesn't produce a curve) or point 1 (this is just the remaining curve we add at the end)
        if point <= &0.01 || point >= &0.99 { continue; }

        // The offset is between remaining_t and 1
        let t = (point - remaining_t) / (1.0-remaining_t);

        // Subdivide the remaining curve at this point
        let (left_curve, right_curve) = remaining.subdivide(t);

        // Work out the offset at this point
        let left_len    = curve.estimate_length(*point);
        let offset      = (final_offset-initial_offset)*(left_len/overall_length) + initial_offset;

        // Add the left curve to the result
        curves_and_offsets.push((left_curve, offset));

        // Update the remaining curve according to the offset
        remaining   = right_curve;
        remaining_t = *point;
    }

    // Add the final remaining curve
    curves_and_offsets.push((remaining, final_offset));

    curves_and_offsets
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
fn simple_offset<Curve: NormalCurve>(curve: Curve, initial_offset: f64, final_offset: f64) -> (Curve, f64) 
where Curve::Point: Normalize<Point=Curve::Point> {
    // Fetch the original points
    let start       = curve.start_point();
    let end         = curve.end_point();
    let (cp1, cp2)  = curve.control_points();

    // The start and end CPs define the curve tangents at the start and end
    let normal_start    = Curve::Point::to_normal(&start, &(cp1-start));
    let normal_end      = Curve::Point::to_normal(&end, &(end-cp2));
    let normal_start    = Curve::Point::from_components(&normal_start).to_unit_vector();
    let normal_end      = Curve::Point::from_components(&normal_end).to_unit_vector();

    // Offset start & end by the specified amounts to create the first approximation of a curve
    // TODO: scale rather than just move for better accuracy
    let new_start   = start + (normal_start * initial_offset);
    let new_cp1     = cp1 + (normal_start * initial_offset);
    let new_cp2     = cp2 + (normal_end * final_offset);
    let new_end     = end + (normal_end * final_offset);

    let mut offset_curve = Curve::from_points(new_start, new_end, new_cp1, new_cp2);

    // Tweak the curve at some sample points to improve the accuracy of our guess
    for sample_t in [0.5, 0.25, 0.75].into_iter() {
        let sample_t = *sample_t;

        // Work out th error at this point
        let move_offset = offset_error(&curve, &offset_curve, sample_t, initial_offset, final_offset);

        // Adjust the curve by the offset
        offset_curve = move_point(&offset_curve, sample_t, move_offset);
    }

    // Use the offset at the curve's midway point as the error
    let error_offset    = offset_error(&curve, &offset_curve, 0.5, initial_offset, final_offset);
    let error           = Curve::Point::origin().distance_to(&error_offset);

    (offset_curve, error)
}
