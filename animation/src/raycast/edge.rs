use super::super::traits::*;

use std::iter;
use std::sync::*;

///
/// The type of a particular vector edge (defines how it interacts with the ray)
///
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RaycastEdgeKind {
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
    pub kind: RaycastEdgeKind,
}

impl RaycastEdge {
    ///
    /// Retrieves the edges corresponding to a particular vector object (when drawn with the specified vector properties)
    ///
    pub fn from_vector<'a>(vector: &'a Vector, properties: Arc<VectorProperties>) -> Box<dyn 'a+Iterator<Item=RaycastEdge>> {
        match vector {
            Vector::BrushDefinition(_defn)      => { Box::new(iter::empty()) }
            Vector::BrushProperties(_props)     => { Box::new(iter::empty()) }

            Vector::Transformed(transform)      => { Self::from_transformed(transform, properties) }
            Vector::BrushStroke(brush_stroke)   => { unimplemented!(); }
            Vector::Path(path)                  => { Self::from_path_element(path) }
        }
    }

    ///
    /// Retrieves the edges corresponding to a transformed element
    ///
    pub fn from_transformed<'a>(vector: &'a TransformedVector, properties: Arc<VectorProperties>) -> Box<dyn 'a+Iterator<Item=RaycastEdge>> {
        // The transformed vector here is an Arc, so we can borrow it for long enough
        // But this requires both the reference and its borrow to live in the same place, and Rust does not support that
        // So transformed vectors will run slowly as we have to store them in a temporary Vec to get around this
        let transformed_vector  = vector.transformed_vector();
        let edges               = Self::from_vector(&*transformed_vector, properties);
        let edge_collection     = edges.collect::<Vec<_>>();

        Box::new(edge_collection.into_iter())
    }

    ///
    /// Retrieves the edges corresponding to a path element
    ///
    pub fn from_path_element<'a>(vector: &'a PathElement) -> Box<dyn 'a+Iterator<Item=RaycastEdge>> {
        match vector.brush().drawing_style() {
            BrushDrawingStyle::Erase    => { Self::from_path(vector.path(), RaycastEdgeKind::EraseContents) }
            BrushDrawingStyle::Draw     => { Self::from_path(vector.path(), RaycastEdgeKind::Solid) }
        }
    }

    ///
    /// Returns the edges for a particular path as ray cast edges
    ///
    pub fn from_path<'a>(path: &'a Path, edge_kind: RaycastEdgeKind) -> Box<dyn 'a+Iterator<Item=RaycastEdge>> {
        Box::new(path.to_curves()
            .map(move |curve| {
                RaycastEdge {
                    curve: curve,
                    kind: edge_kind
                }
            }))
    }
}
