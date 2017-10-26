///
/// Represents a 2D affine transformation matrix
///
#[derive(Clone, Copy, PartialEq)]
pub struct Transform2D((f32, f32, f32), (f32, f32, f32), (f32, f32, f32));
