///
/// Ways that a set of elements can be aligned relative to one another
///
/// 'Middle' and 'Center will align to the anchor point, the others will align to the overall bounding box of the elements
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ElementAlign {
    Left, Center, Right,
    Top, Middle, Bottom
}