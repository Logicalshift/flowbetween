use super::*;

///
/// Creates a bezier curve that fits a set of points with a particular error
/// 
/// Algorithm from Philip J. Schdeiner, Graphics Gems
/// 
pub fn fit_curve<Point: Coordinate, Curve: BezierCurve<Point=Point>>(points: &Vec<Point>, max_error: f32) -> Option<Vec<Curve>> {
    // Need at least 2 points to fit anything
    if points.len() < 2 {
        // Insufficient points for this curve
        None
    } else {
        // Need the start and end tangents so we know how the curve continues
        let start_tangent   = start_tangent(points);
        let end_tangent     = end_tangent(points);

        // Pass on to the main curve fitting algorithm
        Some(fit_curve_cubic(&points[0..points.len()], &start_tangent, &end_tangent, max_error))
    }
}

///
/// Fits a bezier curve to a subset of points
/// 
pub fn fit_curve_cubic<Point: Coordinate, Curve: BezierCurve<Point=Point>>(points: &[Point], start_tangent: &Point, end_tangent: &Point, max_error: f32) -> Vec<Curve> {
    if points.len() <= 2 {
        // 2 points is a line (less than 2 points is an error here)
        fit_line(&points[0], &points[1])
    } else {
        let chords = chords_for_points(points);

        unimplemented!()
    }
}

///
/// Creates a curve representing a line between two points
/// 
fn fit_line<Point: Coordinate, Curve: BezierCurve<Point=Point>>(p1: &Point, p2: &Point) -> Vec<Curve> {
    // Any bezier curve where the control points line up forms a straight line; we use points around 1/3rd of the way along in our generation here
    let direction   = *p2 - *p1;
    let cp1         = *p1 + (direction * 0.33);
    let cp2         = *p1 + (direction * 0.66);

    vec![Curve::from_points(*p1, *p2, cp1, cp2)]
}

///
/// Chord-length parameterizes a set of points
/// 
/// This is an estimate of the 't' value for these points on the final curve.
/// 
fn chords_for_points<Point: Coordinate>(points: &[Point]) -> Vec<f32> {
    let mut distances       = vec![];
    let mut total_distance  = 0.0;

    // Compute the distances for each point
    distances.push(total_distance);
    for p in 1..points.len() {
        total_distance += points[p-1].distance_to(&points[p]);
        distances.push(total_distance);
    }

    // Normalize to the range 0..1
    for p in 0..points.len() {
        distances[p] /= total_distance;
    }

    distances
}

///
/// Returns the unit tangent at the start of the curve
/// 
fn start_tangent<Point: Coordinate>(points: &Vec<Point>) -> Point {
    (points[0]-points[1]).normalize()
}

///
/// Returns the unit tangent at the end of the curve
/// 
fn end_tangent<Point: Coordinate>(points: &Vec<Point>) -> Point {
    (points[points.len()-1]-points[points.len()-2]).normalize()
}
