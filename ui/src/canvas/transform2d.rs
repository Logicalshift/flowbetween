///
/// Represents a 2D affine transformation matrix
///
#[derive(Clone, Copy, PartialEq)]
pub struct Transform2D(pub (f32, f32, f32), pub (f32, f32, f32), pub (f32, f32, f32));
