use super::super::traits::*;

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
    curve: PathCurve,

    /// True if this edge is part of a shape that hides anything beneath it (false if it's an actual edge of a shape)
    hides_beneath: bool
}

impl RaycastEdge {
    ///
    /// Retrieves the edges corresponding to a particular vector object
    ///
    pub fn from_vector(vector: &Vector) -> Box<dyn Iterator<Item=RaycastEdge>> {
        match vector {
            Vector::Transformed(transform) => { unimplemented!(); }
            Vector::BrushDefinition(defn) => { unimplemented!(); }
            Vector::BrushProperties(props) => { unimplemented!(); }
            Vector::BrushStroke(brush_stroke) => { unimplemented!(); }
            Vector::Path(path) => { unimplemented!(); }
        }
    }
}