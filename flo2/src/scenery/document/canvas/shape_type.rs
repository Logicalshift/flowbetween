use ::serde::*;

///
/// Represents the type of a shape
///
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ShapeType(usize);
