use super::super::traits::*;

use std::iter;

pub enum EdgeKind {
    // Solid edge
    Solid,

    // Edge that hides whatever is beneath it
    EraseContents
}

///
/// Represents a raycasting edge
///
pub struct RaycastEdge {
    /// The curve representing this particular edge
    pub curve: PathCurve,

    /// The type of this edge
    pub kind: EdgeKind,
}

impl RaycastEdge {
    ///
    /// Retrieves the edges corresponding to a particular vector object
    ///
    pub fn from_vector(vector: &Vector) -> Box<dyn Iterator<Item=RaycastEdge>> {
        match vector {
            Vector::BrushDefinition(_defn)      => { Box::new(iter::empty()) }
            Vector::BrushProperties(_props)     => { Box::new(iter::empty()) }

            Vector::Transformed(transform)      => { Self::from_transformed(transform) }
            Vector::BrushStroke(brush_stroke)   => { unimplemented!(); }
            Vector::Path(path)                  => { unimplemented!(); }
        }
    }

    ///
    /// Retrieves the edges corresponding to a transformed element
    ///
    pub fn from_transformed(vector: &TransformedVector) -> Box<dyn Iterator<Item=RaycastEdge>> {
        Self::from_vector(&*vector.transformed_vector())
    }
}