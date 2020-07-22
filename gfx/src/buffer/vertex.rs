///
/// 2D vertex representation
///
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vertex2D {
    pos:        [f32; 2],
    tex_coord:  [f32; 2],
    color:      [u8; 4]
}
