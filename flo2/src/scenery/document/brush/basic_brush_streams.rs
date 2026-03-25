use super::brush_point::*;

use futures::prelude::*;

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
