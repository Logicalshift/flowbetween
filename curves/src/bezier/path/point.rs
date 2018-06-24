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
        // TODO: we may need to exclude the first intersection if it's at 0 and the final intersection is at 1
        let mut hit_end_last_pass   = false;

        for curve in path_to_curves::<_, Curve<_>>(path) {
            let mut hit_end_this_pass = false;

            for t in curve_intersects_line(&curve, &ray) {
                // Intersections at t = 1.0 are at the end of the curve.
                if t > 0.9999999 { hit_end_this_pass = true; }

                // Don't treat both the start of a curve and the end of a curve as two separate intersections (it's probably the same one caused by a floating point imprecision)
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
        }

        // Point is inside the path if the ray crosses more lines facing in a particular direction
        total_direction != 0
    }
}
