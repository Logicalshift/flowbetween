use super::vector::*;
use super::element::*;
use super::properties::*;
use super::control_point::*;

use super::super::edit::*;
use super::super::path::*;
use super::super::motion::*;

use canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Represents a vector element that has been transformed from one type of element to another
/// 
#[derive(Clone)]
pub struct TransformedVector {
    /// The vector element before transformations were applied
    original:       Arc<Vector>,

    /// The vector element as it is after the transformations were applied
    transformed:    Arc<Vector>
}

impl VectorElement for TransformedVector {
    ///
    /// The ID of this element
    /// 
    fn id(&self) -> ElementId {
        unimplemented!()
    }

    ///
    /// Retrieves the paths for this element, if there are any
    /// 
    fn to_path(&self, properties: &VectorProperties) -> Option<Vec<Path>> {
        unimplemented!()
    }

    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties) {
        unimplemented!()
    }

    ///
    /// Returns the properties to use for future elements
    /// 
    fn update_properties(&self, properties: Arc<VectorProperties>) -> Arc<VectorProperties> { properties }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    /// 
    fn motion_transform(&self, motion: &Motion, when: Duration) -> Vector {
        unimplemented!()
    }

    ///
    /// Fetches the control points for this element
    /// 
    fn control_points(&self) -> Vec<ControlPoint> {
        unimplemented!()
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    /// 
    /// The vector here specifies the updated position for each control point in control_points
    /// 
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>) -> Vector {
        unimplemented!()
    }
}
