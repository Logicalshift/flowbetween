use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::path_conversion_options::*;
use super::super::path::*;
use super::super::edit::*;

use flo_canvas::*;

use std::time::Duration;
use std::sync::*;
use std::any::*;

///
/// Represents a vector element in a frame
///
pub trait VectorElement : Send+Any {
    ///
    /// The ID of this element
    ///
    fn id(&self) -> ElementId;

    ///
    /// Modifies this element to have a new ID
    ///
    fn set_id(&mut self, new_id: ElementId);

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    fn to_path(&self, properties: &VectorProperties, options: PathConversion) -> Option<Vec<Path>>;

    ///
    /// Renders this vector element
    ///
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, when: Duration);

    ///
    /// Returns the properties to use for future elements
    ///
    fn update_properties(&self, properties: Arc<VectorProperties>, _when: Duration) -> Arc<VectorProperties> { properties }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self, properties: &VectorProperties) -> Vec<ControlPoint>;

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>, properties: &VectorProperties) -> Vector;
}
