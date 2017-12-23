use super::*;

///
/// Computes a series of curves that approximate an offset curve from the specified origin curve
/// 
pub fn offset<Point: Coordinate+Normalize<Point>, Curve: BezierCurve<Point=Point>+NormalCurve<Curve>>(curve: &Curve, initial_offset: f32, final_offset: f32) -> Vec<Curve> {
    // Split the curve at its extremities to generate a set of simpler curves
    let extremities = curve.find_extremities();
    let curves      = split_offsets(curve, initial_offset, final_offset, &extremities);

    // Offset the curves
    let mut offset_curves   = vec![];
    let mut previous_offset = initial_offset;

    for (curve, next_offset) in curves {
        // Offset this curve
        let offset_curve = simple_offset(curve, previous_offset, next_offset);
        offset_curves.push(offset_curve);

        // This is the initial offset of the next curve
        previous_offset = next_offset;
    }

    // TODO: check the offset curve against an error bound and subdivide further if it doesn't make it
    // This is the final result
    offset_curves
}

///
/// Splits a curve at a given set of ordered offsets, returning a list of curves and
/// their final offsets
/// 
fn split_offsets<Point: Coordinate+Normalize<Point>, Curve: BezierCurve<Point=Point>+NormalCurve<Curve>>(curve: &Curve, initial_offset: f32, final_offset: f32, split_points: &[f32]) -> Vec<(Curve, f32)> {
    let mut curves_and_offsets  = vec![];
    let mut remaining           = curve.clone();
    let mut remaining_t         = 0.0;
    
    let overall_length          = curve.estimate_length(1.0);

    for point in split_points {
        // Don't subdivide at point 0 (it doesn't produce a curve) or point 1 (this is just the remaining curve we add at the end)
        if point <= &0.0 || point >= &1.0 { continue; }

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
/// Offsets the endpoints and mid-point of a curve by the specified amounts without subdividing
/// 
/// This won't produce an accurate offset if the curve doubles back on itself
/// 
fn simple_offset<Point: Coordinate+Normalize<Point>, Curve: BezierCurve<Point=Point>+NormalCurve<Curve>>(curve: Curve, initial_offset: f32, final_offset: f32) -> Curve {
    // Fetch the original points
    let start       = curve.start_point();
    let end         = curve.end_point();
    let (cp1, cp2)  = curve.control_points();

    // The start and end CPs define the curve tangents at the start and end
    let normal_start    = Point::to_normal(&start, &(cp1-start));
    let normal_end      = Point::to_normal(&end, &(end-cp2));
    let normal_start    = Point::from_components(&normal_start).to_unit_vector();
    let normal_end      = Point::from_components(&normal_end).to_unit_vector();

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

        // Work out how much we need to offset the mid-point
        let midpoint_offset     = (final_offset - initial_offset) * (curve.estimate_length(sample_t)/curve.estimate_length(1.0)) + initial_offset;
        let midpoint_normal     = curve.normal_at_pos(sample_t).to_unit_vector();
        let original_midpoint   = curve.point_at_pos(sample_t);
        let new_midpoint        = offset_curve.point_at_pos(sample_t);
        let target_pos          = original_midpoint + midpoint_normal*midpoint_offset;
        let move_offset         = target_pos - new_midpoint;

        // Adjust the curve by the offset
        offset_curve = move_point(&offset_curve, sample_t, move_offset);
    }

    offset_curve
}
