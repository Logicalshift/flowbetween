use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::super::path::*;
use super::super::edit::*;
use super::super::motion::*;

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
    /// Retrieves the paths for this element, if there are any
    /// 
    fn to_path(&self, properties: &VectorProperties) -> Option<Vec<Path>>;

    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, when: Duration);

    ///
    /// Returns the properties to use for future elements
    /// 
    fn update_properties(&self, properties: Arc<VectorProperties>) -> Arc<VectorProperties> { properties }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    /// 
    fn motion_transform(&self, motion: &Motion, when: Duration) -> Vector;

    ///
    /// Fetches the control points for this element
    /// 
    fn control_points(&self) -> Vec<ControlPoint>;

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    /// 
    /// The vector here specifies the updated position for each control point in control_points
    /// 
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>) -> Vector;
}
