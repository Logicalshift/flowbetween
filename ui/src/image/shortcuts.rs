use super::Image;
use super::static_data::*;

use std::sync::*;

///
/// Returns a PNG image built from static data
///
pub fn png_static(data: &'static [u8]) -> Image {
    Image::Png(Arc::new(StaticImageData::new(data)))
}

///
/// Return a SVG image built from static data
///
pub fn svg_static(data: &'static [u8]) -> Image {
    Image::Svg(Arc::new(StaticImageData::new(data)))
}
