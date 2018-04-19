use nanovg::*;

///
/// Possible NanoVgPaint instructions
///
pub enum NanoVgPaint {
    Color(Color),
    Gradient(Gradient)
}
