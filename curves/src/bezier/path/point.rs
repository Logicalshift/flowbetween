use super::path::*;
use super::to_curves::*;
use super::super::curve::*;
use super::super::normal::*;
use super::super::intersection::*;
use super::super::super::coordinate::*;

///
/// Returns true if a particular point is within a bezier path
/// 
pub fn path_contains_point<P: BezierPath>(path: &P, point: &P::Point) -> bool
where P::Point: Coordinate2D {
    // We want to cast a ray from the outer edge of the bounds to our point
    let (min_bounds, max_bounds) = path.bounding_box();

    if min_bounds.x() > point.x() || max_bounds.x() < point.x() || min_bounds.y() > point.y() || max_bounds.y() < point.y() {
        // Point is outside the bounds of the path
        false
    } else {
        // Ray is from the top of the bounds to our point
        let ray             = (max_bounds + P::Point::from_components(&[0.01, 0.01]), *point);
        let ray_direction   = ray.1 - ray.0;

        // The total of all of the ray directions
        let mut total_direction     = 0;

        // Whether or not we hit the end this pass
        let mut hit_end_last_pass   = false;

        // True if we're on the first path
        let mut first_path          = true;
        let mut hit_first_point     = false;

        // Generate the set of curves for this path
        let curves                  = path_to_curves::<_, Curve<_>>(path);
        let mut curves              = curves.into_iter().peekable();

        while let Some(curve) = curves.next() {
            let mut hit_end_this_pass = false;

            for (t, _pos) in curve_intersects_line(&curve, &ray) {
                // If we precisely hit the first point, we need to make sure we don't also precisely hit the last point
                if t < 0.0000001 && first_path { hit_first_point = true; }

                // Hitting both the end and first point precisely should count as hitting only the first point
                if t > 0.9999999 && hit_first_point && curves.peek().is_none() { continue; }

                // Intersections at t = 1.0 are at the end of the curve.
                if t > 0.9999999 { hit_end_this_pass = true; }

                // If we hit the start point of this curve after hitting the end point of the preceding curve, assume that we've just received the same hit twice
                if t < 0.0000001 && hit_end_last_pass { continue; }

                // Get the normal at this point
                let normal = curve.normal_at_pos(t);

                // Dot product determines the direction of the normal relative to the ray (+ve if in the same direction or -ve in the opposite)
                // That is, the sign of this calculation indicates which direction the line is facing.
                // One of these directions is 'entering' the shape and one is 'leaving': if we leave as often as we enter, the point is inside
                // (We don't actually need to worry which is which here as we know the ray starts outside of the curve)
                let direction = normal.dot(&ray_direction);

                if direction < 0.0 {
                    total_direction -= 1;
                } else if direction > 0.0 {
                    total_direction += 1;
                }
            }

            // Pass on whether or not we hit the end of the curve during this pass
            hit_end_last_pass = hit_end_this_pass;

            // No longer the first path
            first_path = false;
        }

        // Point is inside the path if the ray crosses more lines facing in a particular direction
        total_direction != 0
    }
}
