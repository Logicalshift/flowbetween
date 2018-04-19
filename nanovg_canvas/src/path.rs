///
/// NanoVg path instruction
/// 
pub enum NanoVgPath {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    CubicBezier((f32, f32), (f32, f32), (f32, f32)),
    Fill()   
}