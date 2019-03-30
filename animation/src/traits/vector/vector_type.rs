///
/// The type of a vector element
///
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VectorType {
    /// Vector element representing the definition of a brush
    BrushDefinition,

    /// Vector element representing the properties of a brush
    BrushProperties,

    /// Vector element representing a brush stroke
    BrushStroke,

    /// Vector element representing a path
    Path
}
