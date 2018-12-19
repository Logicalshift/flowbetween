use super::element::*;
use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::super::path::*;
use super::super::edit::*;
use super::super::motion::*;

use canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Element representing a path definition
///
#[derive(Clone)]
pub struct PathElement {
    /// The ID of this path
    id: ElementId,

    /// The components that make up this path
    components: Arc<Vec<PathComponent>>,

    /// The properties of this path
    properties: Arc<VectorProperties>
}

impl VectorElement for PathElement {
    ///
    /// The ID of this element
    /// 
    fn id(&self) -> ElementId { 
        self.id
    }

    ///
    /// Retrieves the paths for this element, if there are any
    /// 
    fn to_path(&self, _properties: &VectorProperties) -> Option<Vec<Path>> { 
        Some(vec![Path::from_elements(self.components.iter().map(|component| *component))])
    }

    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties) { unimplemented!() }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    /// 
    fn motion_transform(&self, motion: &Motion, when: Duration) -> Vector { unimplemented!() }

    ///
    /// Fetches the control points for this element
    /// 
    fn control_points(&self) -> Vec<ControlPoint> { unimplemented!() }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    /// 
    /// The vector here specifies the updated position for each control point in control_points
    /// 
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>) -> Vector { unimplemented!() }
}
