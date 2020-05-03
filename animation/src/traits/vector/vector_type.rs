use super::vector::*;

///
/// The type of a vector element
///
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum VectorType {
    /// Vector element representing the definition of a brush
    BrushDefinition,

    /// Vector element representing the properties of a brush
    BrushProperties,

    /// Vector element representing a brush stroke
    BrushStroke,

    /// Vector element representing a path
    Path,

    /// Vector element representing the way something moves through space
    Motion,

    /// Group of other vector elements
    Group,

    /// A property describing a transformation that can be applied to another element
    Transformation,

    /// Element that exists but could not be loaded
    Error
}

impl From<&Vector> for VectorType {
    fn from(vector: &Vector) -> VectorType {
        use self::Vector::*;

        match vector {
            Transformed(transformed_from)   => VectorType::from(&*transformed_from.without_transformations()),
            BrushDefinition(_)              => VectorType::BrushDefinition,
            BrushProperties(_)              => VectorType::BrushProperties,
            BrushStroke(_)                  => VectorType::BrushStroke,
            Path(_)                         => VectorType::Path,
            Motion(_)                       => VectorType::Motion,
            Group(_)                        => VectorType::Group,
            Transformation(_)               => VectorType::Transformation,
            Error                           => VectorType::Error
        }
    }
}
