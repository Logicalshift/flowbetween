use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::vector_element::*;
use super::path_conversion_options::*;

use super::super::edit::*;
use super::super::path::*;
use super::super::motion::*;

use flo_canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Represents a vector element that has been transformed from one type of element to another
///
#[derive(Clone, Debug)]
pub struct TransformedVector {
    /// The vector element before transformations were applied
    original:       Arc<Vector>,

    /// The vector element as it is after the transformations were applied
    transformed:    Arc<Vector>
}

impl TransformedVector {
    ///
    /// Creates a new transformed vector
    ///
    pub fn new(original: Vector, transformed: Vector) -> TransformedVector {
        TransformedVector {
            original: Arc::new(original),
            transformed: Arc::new(transformed)
        }
    }

    ///
    /// Returns the original vector for this transformed vector without any transformations applied to it
    ///
    pub fn without_transformations(&self) -> Arc<Vector> {
        Arc::clone(&self.original)
    }

    ///
    /// Returns the transformed shape of this vector
    ///
    pub fn transformed_vector(&self) -> Arc<Vector> {
        Arc::clone(&self.transformed)
    }
}

impl VectorElement for TransformedVector {
    ///
    /// The ID of this element
    ///
    #[inline]
    fn id(&self) -> ElementId {
        self.transformed.id()
    }

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    #[inline]
    fn to_path(&self, properties: &VectorProperties, options: PathConversion) -> Option<Vec<Path>> {
        self.transformed.to_path(properties, options)
    }

    ///
    /// Renders this vector element
    ///
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, when: Duration) {
        self.transformed.render(gc, properties, when)
    }

    ///
    /// Returns the properties to use for future elements
    ///
    #[inline]
    fn update_properties(&self, properties: Arc<VectorProperties>) -> Arc<VectorProperties> {
        self.transformed.update_properties(properties)
    }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    ///
    fn motion_transform(&self, motion: &Motion, when: Duration) -> Vector {
        let transformed = self.transformed.motion_transform(motion, when);

        Vector::Transformed(match transformed {
            Vector::Transformed(transformed) => TransformedVector {
                original:       Arc::clone(&self.original),
                transformed:    Arc::clone(&transformed.transformed)
            },

            transformed => TransformedVector {
                original:       Arc::clone(&self.original),
                transformed:    Arc::new(transformed)
            }
        })
    }

    ///
    /// Fetches the control points for this element
    ///
    #[inline]
    fn control_points(&self) -> Vec<ControlPoint> {
        self.transformed.control_points()
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    #[inline]
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>) -> Vector {
        Vector::Transformed(TransformedVector {
            original:       Arc::clone(&self.original),
            transformed:    Arc::new(self.transformed.with_adjusted_control_points(new_positions))
        })
    }
}
