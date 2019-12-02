use super::vector::*;
use super::element::*;
use super::properties::*;
use super::control_point::*;
use super::super::edit::*;
use super::super::path::*;
use super::super::motion::*;

use flo_canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// The motion element describes a Motion as a vector element
///
#[derive(Clone, Debug)]
pub struct MotionElement {
    id:     ElementId,
    motion: Arc<Motion>
}

impl MotionElement {
    ///
    /// Creates a new motion element
    ///
    pub fn new(id: ElementId, motion: Motion) -> MotionElement {
        MotionElement {
            id:         id,
            motion:     Arc::new(motion)
        }
    }
}

impl VectorElement for MotionElement {
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
        // Not a path element
        None
    }

    ///
    /// Renders this vector element
    ///
    fn render(&self, _gc: &mut dyn GraphicsPrimitives, _properties: &VectorProperties, _when: Duration) {
        // Nothing to do
    }

    ///
    /// Returns the properties to use for future elements
    ///
    fn update_properties(&self, properties: Arc<VectorProperties>) -> Arc<VectorProperties> {
        // Clone the properties
        let mut properties          = (*properties).clone();
        let motion                  = Arc::clone(&self.motion);

        // Add a transformation for this motion
        let old_transform_vector    = properties.transform_vector;
        properties.transform_vector = Arc::new(move |vector, when| {
            let vector  = old_transform_vector(vector, when);
            let vector  = vector.motion_transform(&*motion, when);

            vector
        });

        Arc::new(properties)
    }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    ///
    fn motion_transform(&self, _motion: &Motion, _when: Duration) -> Vector {
        Vector::Motion(self.clone())
    }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self) -> Vec<ControlPoint> {
        // TODO: can return control points to make this motion editable here
        vec![]
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    fn with_adjusted_control_points(&self, _new_positions: Vec<(f32, f32)>) -> Vector {
        // TODO: can update time points from control points if we want
        Vector::Motion(self.clone())
    }
}
