use flo_ui::*;
use flo_animation::*;

///
/// Converts a UI Painting struct to a BrushPoint
///
pub fn raw_point_from_painting(painting: &Painting) -> RawPoint {
    RawPoint {
        position:   painting.location,
        tilt:       (painting.tilt_x, painting.tilt_y),
        pressure:   painting.pressure
    }
}
