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

    ///
    /// Retrieves the motion represented by this element
    ///
    pub fn motion(&self) -> Arc<Motion> {
        Arc::clone(&self.motion)
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
    /// Modifies this element to have a new ID
    ///
    fn set_id(&mut self, new_id: ElementId) {
        self.id = new_id
    }

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    fn to_path(&self, _properties: &VectorProperties, _options: PathConversion) -> Option<Vec<Path>> {
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
    fn update_properties(&self, properties: Arc<VectorProperties>, when: Duration) -> Arc<VectorProperties> {
        // Get the transformation for this motion
        let transform       = self.motion.transformation(when);

        if transform.len() > 0 {
            // Add the transform to the properties
            let mut properties      = (*properties).clone();
            let mut full_transform  = (*properties.transformations).clone();

            full_transform.extend(transform);

            properties.transformations = Arc::new(full_transform);

            Arc::new(properties)
        } else {
            // Keep the properties as they were
            properties
        }
    }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self, _properties: &VectorProperties) -> Vec<ControlPoint> {
        // TODO: can return control points to make this motion editable here
        vec![]
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    fn with_adjusted_control_points(&self, _new_positions: Vec<(f32, f32)>, _properties: &VectorProperties) -> Vector {
        // TODO: can update time points from control points if we want
        Vector::Motion(self.clone())
    }
}
