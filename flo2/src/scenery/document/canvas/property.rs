use ::serde::*;

///
/// Identifier for a canvas property
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasPropertyId(usize);

///
/// Value of a specific property set on a shape, layer or brush
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub enum CanvasProperty {
    /// Property with a single float value
    Float(f64),

    /// Property with a single integer value
    Int(i64),

    /// Property with a value that's a floating point number
    FloatList(Vec<f64>),

    /// Property with a value that's a list of integers
    IntList(Vec<i64>),

    /// Property with a value that's a series of bytes
    ByteList(Vec<u8>),
}
