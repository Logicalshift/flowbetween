use flo_curves::*;
use flo_curves::bezier::path::*;

///
/// Returns true if a particular point is within a bezier path
/// 
pub fn point_is_in_path<P: BezierPath>(path: &Vec<P>, point: &P::Point) -> bool
where P::Point: Coordinate2D {
    // The GraphPath has the functionality we need but in order to use it, we need to convert the path as passed in, which is not particularly efficient
    let graph_path              = GraphPath::from_merged_paths(path.iter().map(|path| (path, PathDirection::from(path))));

    // Cast a ray towards the point (direction doesn't matter, so we'll cast at a 45-degree angle
    let ray                     = (*point - P::Point::from_components(&[1.0, 1.0]), *point);
    // let ray_direction           = ray.1 - ray.0;
    let collisions              = graph_path.ray_collisions(&ray);

    // Total direction is 0 when the ray is outside of the path
    let mut total_direction     = 0;

    for (_collision, _curve_t, ray_t, _position) in collisions {
        // Stop once we reach the target point
        if ray_t > 1.0 { break; }

        /* -- TODO: while this does give us a non-zero winding rule, it seems to fail for things like brush strokes that overlap themselves
        // Get the edge where the collision occured
        let edge_ref        = match collision {
            GraphRayCollision::SingleEdge(edge_ref)     => edge_ref,
            GraphRayCollision::Intersection(edge_ref)   => edge_ref
        };
        let edge            = graph_path.get_edge(edge_ref);

        // The direction of the normal and the direction of the path determines the change in the total direction (ie, use a non-zero winding rule)
        let normal          = edge.normal_at_pos(curve_t);
        let direction       = ray_direction.dot(&normal).signum() as i32;
        let path_direction  = edge.label();
        let direction       = if path_direction == PathDirection::Anticlockwise { -direction } else { direction };

        total_direction     += direction;
        */

        // Even-odd path detection
        total_direction     = if total_direction == 0 { 1 } else { 0 };
    }

    // Point is inside the path if the ray has crossed an odd number of lines
    total_direction != 0
}

///
/// Returns true if a particular point is within a bezier path, along with the distance to the nearest edge
/// 
pub fn point_is_in_path_with_distance<P: BezierPath>(path: &Vec<P>, point: &P::Point) -> (bool, f64)
where P::Point: Coordinate2D {
    // The GraphPath has the functionality we need but in order to use it, we need to convert the path as passed in, which is not particularly efficient
    let graph_path              = GraphPath::from_merged_paths(path.iter().map(|path| (path, PathDirection::from(path))));

    // Cast a ray towards the point (direction doesn't matter, so we'll cast at a 45-degree angle
    let ray                     = (*point - P::Point::from_components(&[1.0, 1.0]), *point);
    // let ray_direction           = ray.1 - ray.0;
    let collisions              = graph_path.ray_collisions(&ray);

    // Total direction is 0 when the ray is outside of the path
    let mut total_direction     = 0;
    let mut min_distance_sq     = f64::MAX;

    for (_collision, _curve_t, ray_t, position) in collisions {
        // Track the distance of this collision from the point
        let dx          = point.x() - position.x();
        let dy          = point.y() - position.y();
        let distance_sq = dx*dx + dy*dy;

        if distance_sq < min_distance_sq {
            min_distance_sq = distance_sq;
        }

        // Stop once we reach the target point
        if ray_t > 1.0 { break; }

        // Even-odd path detection
        total_direction     = if total_direction == 0 { 1 } else { 0 };
    }

    // Point is inside the path if the ray has crossed an odd number of lines
    (total_direction != 0, min_distance_sq.sqrt())
}
