use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::vector_element::*;
use super::path_conversion_options::*;
use super::super::path::*;
use super::super::edit::*;

use flo_canvas::*;

use std::sync::*;
use std::time::Duration;

lazy_static! {
    pub (super) static ref ERROR_ELEMENT: ErrorElement = ErrorElement;
}

///
/// Represents an element that could not be deserialized
///
pub struct ErrorElement;

impl VectorElement for ErrorElement {
    ///
    /// The ID of this element
    ///
    fn id(&self) -> ElementId {
        ElementId::Unassigned
    }

    ///
    /// Modifies this element to have a new ID
    ///
    fn set_id(&mut self, _new_id: ElementId) {
    }

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    fn to_path(&self, _properties: &VectorProperties, _options: PathConversion) -> Option<Vec<Path>> {
        None
    }

    ///
    /// Updates the vector properties for future elements
    ///
    fn update_properties(&self, properties: Arc<VectorProperties>, _when: Duration) -> Arc<VectorProperties> {
        properties
    }

    ///
    /// Renders this vector element
    ///
    fn render(&self, _gc: &mut dyn GraphicsPrimitives, _properties: &VectorProperties, _when: Duration) {
    }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self, _proprties: &VectorProperties) -> Vec<ControlPoint> {
        vec![]
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    fn with_adjusted_control_points(&self, _new_positions: Vec<(f32, f32)>, _properties: &VectorProperties) -> Vector {
        Vector::Error
    }
}
