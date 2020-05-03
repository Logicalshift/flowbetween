use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::vector_element::*;
use super::path_conversion_options::*;

use crate::traits::edit::*;
use crate::traits::path::*;
use crate::traits::motion::*;

use flo_canvas::*;

use std::sync::*;
use std::time::{Duration};

///
/// Vector element that represents the possible transformations that can be
///
#[derive(Clone, PartialEq, Debug)]
pub enum Transformation {
    /// A 2D transformation matrix
    Matrix([[f64; 3]; 3])
}

impl VectorElement for (ElementId, Transformation) {
    ///
    /// The ID of this element
    ///
    fn id(&self) -> ElementId {
        self.0
    }

    ///
    /// Modifies this element to have a new ID
    ///
    fn set_id(&mut self, new_id: ElementId) {
        self.0 = new_id;
    }

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    fn to_path(&self, _properties: &VectorProperties, _options: PathConversion) -> Option<Vec<Path>> {
        None
    }

    ///
    /// Renders this vector element
    ///
    fn render(&self, _gc: &mut dyn GraphicsPrimitives, _properties: &VectorProperties, _when: Duration) {

    }

    ///
    /// Returns the properties to use for future elements
    ///
    fn update_properties(&self, properties: Arc<VectorProperties>) -> Arc<VectorProperties> { 
        properties
    }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    ///
    fn motion_transform(&self, _motion: &Motion, _when: Duration) -> Vector {
        Vector::Transformation(self.clone())
    }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self) -> Vec<ControlPoint> {
        vec![]
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>) -> Vector {
        Vector::Transformation(self.clone())
    }
}
