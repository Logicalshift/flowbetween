use flo_canvas;
use nanovg::*;

///
/// Possible NanoVgPaint instructions
///
pub enum NanoVgPaint {
    Color(Color),
    Gradient(Gradient)
}

impl From<flo_canvas::Color> for NanoVgPaint {
    fn from(item: flo_canvas::Color) -> NanoVgPaint {
        let (r, g, b, a) = item.to_rgba_components();

        NanoVgPaint::Color(Color::new(r, g, b, a))
    }
}