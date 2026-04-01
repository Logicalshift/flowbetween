use super::generate_images::*;

use flo_ui::*;
use flo_canvas::*;

const HSLUV_LIGHTNESS: f32  = 65.0;
const HSLUV_SATURATION: f32 = 100.0;

///
/// Generats a pixel on a HSLUV colour wheel
///
#[inline]
fn hsluv_pixel(ratio: f64) -> (u8, u8, u8, u8) {
    // h is in the range 0..360
    let h               = (ratio * 360.0) as f32;

    // Convert to RGB
    let hsluv           = Color::Hsluv(h, HSLUV_SATURATION, HSLUV_LIGHTNESS, 1.0);
    let (r, g, b, _)    = hsluv.to_rgba_components();

    // ... and to pixel data
    ((r*255.0) as u8, (g*255.0) as u8, (b*255.0) as u8, 255)
}

lazy_static! {
    /// An image representing the HSLUV colour wheel
    pub static ref HSLUV_COLOR_WHEEL: Image = image_for_wheel_fn(hsluv_pixel, 512, 140, 180.0);
}
