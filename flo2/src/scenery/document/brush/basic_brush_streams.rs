use super::brush_point::*;

use futures::prelude::*;

use flo_curves::geo::*;
use flo_curves::bezier::*;
use flo_stream::*;

///
/// Converts an input stream of brush points to an output stream of brush points with a radius based on the pen pressure
///
pub fn brush_pressure_to_radius_linear(max_radius: f64, input_stream: impl 'static + Send + Stream<Item=BrushPoint>) -> impl 'static + Send + Stream<Item=BrushPoint> {
    input_stream.map(move |point| {
        let mut point = point;

        point.daub_radius = Some((point.pressure.max(0.0).min(1.0)) * max_radius);

        point
    })
}

///
/// Calculates an interpolation between two points in one dimension
///
/// This returns a function that interpolates between x2 and x3 given a value between 0 and 1, where x1 and x4 are the following points
/// We use this for generating intermediate points when we're generating points for a brush stroke that are supposed to be a linear distance apart.
///
/// The tension can be used to adjust the amount of curvature (though 1.0 is a good value)
///
#[inline]
fn interpolate_points(x1: f64, x2: f64, x3: f64, x4: f64, tension: f64) -> impl Fn(f64) -> f64 {
    let cp1 = x2 + (x2-x1)*tension;
    let cp2 = x3 - (x4-x3)*tension;

    move |t| de_casteljau4(t, x2, cp1, cp2, x3)
}

///
/// Calculates an interpolation between two points in one dimension
///
/// This returns a function that interpolates between x2 and x3 given a value between 0 and 1, where x1 and x4 are the following points
/// We use this for generating intermediate points when we're generating points for a brush stroke that are supposed to be a linear distance apart.
///
/// The tension can be used to adjust the amount of curvature (though 1.0 is a good value)
///
#[inline]
fn interpolate_coords<TCoord>(x1: TCoord, x2: TCoord, x3: TCoord, x4: TCoord, tension: f64) -> Curve<TCoord>
where
    TCoord: Coordinate + Coordinate2D,
{
    let cp1 = x2 + (x2-x1)*tension;
    let cp2 = x3 - (x4-x3)*tension;

    Curve::from_points(x2, (cp1, cp2), x3)
}

///
/// Walks between brush points p2 and p3 at a set of fixed distances
///
#[inline]
fn walk_between_brush_points(p1: &BrushPoint, p2: &BrushPoint, p3: &BrushPoint, p4: &BrushPoint, initial_distance: f64, step_distance: f64, tension: f64) -> impl Iterator<Item=BrushPoint> {
    use std::iter;

    // Interpolate the coordinates
    let coords_curve = interpolate_coords(Coord2(p1.position.0, p1.position.1), Coord2(p2.position.0, p2.position.1), Coord2(p3.position.0, p3.position.1), Coord2(p4.position.0, p4.position.1), tension);

    // Also the pressure, tilt, rotation and flow rate. Optional fields are left as None by this routine
    let pressure    = interpolate_points(p1.pressure, p2.pressure, p3.pressure, p4.pressure, tension);
    let rotation    = interpolate_points(p1.rotation, p2.rotation, p3.rotation, p4.rotation, tension);
    let flow_rate   = interpolate_points(p1.flow_rate, p2.flow_rate, p3.flow_rate, p4.flow_rate, tension);
    let tilt_x      = interpolate_points(p1.tilt.0, p2.tilt.0, p3.tilt.0, p4.tilt.0, tension);
    let tilt_y      = interpolate_points(p1.tilt.1, p2.tilt.1, p3.tilt.1, p4.tilt.1, tension);

    // Create the initial walk
    let walk = walk_curve_evenly(&coords_curve, step_distance, 0.01);

    // Vary according to the initial distance and step distance
    let initial_distance    = if initial_distance > 0.0 { Some(initial_distance) } else { None };
    let step_distance       = iter::once(step_distance).cycle();

    let walk = walk.vary_by(initial_distance.into_iter().chain(step_distance));

    // Map the curve sections to brush points
    walk.map(move |section| {
        let t = section.t_for_t(1.0);

        let Coord2(x, y) = coords_curve.point_at_pos(t);

        BrushPoint {
            position:   (x, y),
            pressure:   pressure(t),
            rotation:   rotation(t),
            flow_rate:  flow_rate(t),
            tilt:       (tilt_x(t), tilt_y(t)),

            ..Default::default()
        }
    })
}

///
/// Takes a set of brush points from an input device and smooths them to generate brush points that are a fixed distance apart
///
/// To generate points that are in between the input points, this applies a simple smoothing algorihtm to them. There must be at
/// least 3 input points for this to start generating points after the first point (this is because we need 4 points, but we can
/// generate a fake initial point from the second point)
///
pub fn brush_fill_in_points(distance: f64, input_stream: impl 'static + Send + Stream<Item=BrushPoint>) -> impl 'static + Send + Stream<Item=BrushPoint> {
    generator_stream(move |yield_fn| async move {
        use std::pin::{pin};

        // Pin the stream for reading
        let mut input_stream = pin!(input_stream);

        // Before we can generate any points we need at least 4 points. This is a circular buffer
        let mut previous_points     = [BrushPoint::default(); 4];
        let mut previous_point_idx  = 0;

        // Read the first 3 points. First point is always generated as-is
        let Some(first_point) = input_stream.next().await else { return; };
        previous_points[1] = first_point;

        yield_fn(first_point).await;

        for idx in 2..4 {
            let Some(next_point) = input_stream.next().await else { return; };
            previous_points[idx] = next_point;
        }

        // Generate a fake 'previous' point for point 0 (so we can interpolate between the first and second points, and also so we only need 3 points to prime the algorithm)
        let dx = previous_points[2].position.0 - previous_points[1].position.1;
        let dy = previous_points[2].position.1 - previous_points[1].position.0;

        let fake_point = (first_point.position.0 - dx, first_point.position.1 - dy);
        let fake_point = BrushPoint { position: fake_point, ..Default::default() };

        previous_points[0] = fake_point;

        // We also need to know the last generated point, and the distance we've consumed between any points we might have discarded
        let mut last_generated_point    = first_point;
        let mut distance_covered        = 0.0;

        // Process each point that we get from the stream
        while let Some(next_point) = input_stream.next().await {
            // Indexes of the 4 points in our circular buffer
            let p0_idx = previous_point_idx;
            let p1_idx = (previous_point_idx+1)%4;
            let p2_idx = (previous_point_idx+2)%4;
            let p3_idx = (previous_point_idx+3)%4;

            // Generate points using our existing set of 4 points (we're about to lose the first point and we're generating points between the second and third points)
            let section_distance = previous_points[p1_idx].distance_to(&previous_points[p2_idx]);

            if section_distance + distance_covered >= distance {
                // Return points along this curve
                let initial_distance = distance - distance_covered;

                for point in walk_between_brush_points(&previous_points[p0_idx], &previous_points[p1_idx], &previous_points[p2_idx], &previous_points[p3_idx], initial_distance, distance, 1.0) {
                    yield_fn(point).await;
                    last_generated_point = point;
                }

                // Distance covered is the distance from the last point we generated to the end point of this section
                distance_covered = last_generated_point.distance_to(&previous_points[p2_idx]);
            } else {
                // Just 'cover' this distance linearly and ignore the point
                distance_covered += section_distance;
            }

            // Store this point
            previous_points[previous_point_idx] = next_point;
            previous_point_idx = (previous_point_idx+1)%4;
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn interpolate_1d_start_and_end() {
        let interpolate_fn = interpolate_points(2.0, 3.0, 4.0, 2.0, 1.0);

        assert!(interpolate_fn(0.0) == 3.0, "{} != 3.0", interpolate_fn(0.0));
        assert!(interpolate_fn(1.0) == 4.0, "{} != 4.0", interpolate_fn(1.0));
    }

    #[test]
    fn interpolate_coords_start_and_end() {
        let curve = interpolate_coords(Coord2(2.0, 3.0), Coord2(3.0, 4.0), Coord2(4.0, 4.0), Coord2(2.0, 8.0), 1.0);

        assert!(curve.point_at_pos(0.0) == Coord2(3.0, 4.0), "{:?} != Coord2(3.0, 4.0)", curve.point_at_pos(0.0));
        assert!(curve.point_at_pos(1.0) == Coord2(4.0, 4.0), "{:?} != Coord2(4.0, 4.0)", curve.point_at_pos(1.0));
    }
}