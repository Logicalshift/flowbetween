use flo_canvas::*;

///
/// Attributes used to render a bezier path
///
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AnimationPathAttribute {
    /// Path is drawn as a stroke with the specified width and colour
    Stroke(BlendMode, f32, Color, LineJoin, LineCap),

    /// Path is drawn as a stroke with the specified pixel width and colour
    StrokePixels(BlendMode, f32, Color, LineJoin, LineCap),

    /// Path is filled with the specified colour
    Fill(BlendMode, Color, WindingRule),

    /// Path is filled with the specified texture
    FillTexture(BlendMode, TextureId, (f32, f32), (f32, f32), Option<Transform2D>, WindingRule),

    /// Path is filled with the specified gradient
    FillGradient(BlendMode, GradientId, (f32, f32), (f32, f32), Option<Transform2D>, WindingRule)
}
