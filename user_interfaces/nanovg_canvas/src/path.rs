use nanovg::Path;

///
/// NanoVg path instruction
///
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NanoVgPath {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    CubicBezier((f32, f32), (f32, f32), (f32, f32)),
    Close
}

impl NanoVgPath {
    #[inline]
    pub fn add_to_path(&self, path: &Path) {
        use self::NanoVgPath::*;

        match self {
            &MoveTo(x, y)               => { path.move_to((x, y)); },
            &LineTo(x, y)               => { path.line_to((x, y)); },
            &CubicBezier(pos, cp1, cp2) => { path.cubic_bezier_to(pos, cp1, cp2); },
            &Close                      => { path.close(); }
        }
    }
}
