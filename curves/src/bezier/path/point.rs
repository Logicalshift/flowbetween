use super::path::*;
use super::bounds::*;
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
    let (min_bounds, max_bounds) = path_bounds(path);

    if min_bounds.x() > point.x() || max_bounds.x() < point.x() || min_bounds.y() > point.y() || max_bounds.y() < point.y() {
        // Point is outside the bounds of the path
        false
    } else {
        // Ray is from the top of the bounds to our point
        let ray             = (max_bounds, *point);
        let ray_direction   = ray.1 - ray.0;

        // Compute all the normals of the intersections of the ray with each line in this curve
        let intersection_normals = path_to_curves::<_, Curve<_>>(path)
            .flat_map(|curve| curve_intersects_line(&curve, &ray)
                .into_iter()
                .map(|t| curve.normal_at_pos(t))
                .collect::<Vec<_>>());

        // Use the dot product to determine the normal direction relative to the ray (-1, 0 or 1)
        // This indicates whether or not the line being crossed is facing 'inwards' or 'outwards'
        let directions = intersection_normals
            .map(|normal| normal.dot(&ray_direction))
            .map(|direction| if direction < 0.0 { -1 } else if direction > 0.0 { 1 } else { 0 });
        
        // If the sum of the directions is 0 then the point is outside of the path (crosses a 'leaving' line as often as it crosses an 'entering' line), otherwise it's inside
        directions.sum::<i32>() != 0
    }
}
