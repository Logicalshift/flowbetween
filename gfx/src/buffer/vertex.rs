///
/// 2D vertex representation
///
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vertex2D {
    pub pos:        [f32; 2],
    pub tex_coord:  [f32; 2],
    pub color:      [u8; 4]
}
