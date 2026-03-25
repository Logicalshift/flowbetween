use super::brush_point::*;

use futures::prelude::*;

use flo_curves::geo::*;
use flo_curves::bezier::*;

///
/// Converts an input stream of brush points to an output stream of brush points with a radius based on the pen pressure
///
pub fn brush_pressure_to_radius_linear(max_radius: f64, input_stream: impl 'static + Send + Stream<Item=BrushPoint>) -> impl 'static + Send + Stream<Item=BrushPoint> {
    input_stream.map(move |point| {
        let mut point = point;

        if let Some(pressure) = point.pressure {
            point.daub_radius = Some((pressure.max(0.0).min(1.0)) * max_radius);
        } else {
            point.daub_radius = Some(max_radius);
        }

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

/*
///
/// Takes a set of brush points from an input device and smooths them to generate brush points that are a fixed distance apart
///
/// To generate points that are in between the input points, this applies a simple smoothing algorihtm to them
///
pub fn brush_fill_in_points(distance: f64, input_stream: impl 'static + Send + Stream<Item=BrushPoint>) -> impl 'static + Send + Stream<Item=BrushPoint> {
    todo!()
}
*/

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