use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::vector_element::*;
use super::super::path::*;
use super::super::edit::*;
use super::super::motion::*;
use super::super::brush_properties::*;

use flo_canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Element representing selecting some new brush properties
///
#[derive(Clone, Debug)]
pub struct BrushPropertiesElement {
    /// The ID of this element
    id: ElementId,

    /// The brush properties to set
    new_properties: BrushProperties
}

impl Default for BrushPropertiesElement {
    fn default() -> BrushPropertiesElement {
        BrushPropertiesElement::new(ElementId::Unassigned, BrushProperties::new())
    }
}

impl BrushPropertiesElement {
    ///
    /// Creates a new brush properties vector element
    ///
    pub fn new(id: ElementId, new_properties: BrushProperties) -> BrushPropertiesElement {
        BrushPropertiesElement {
            id:                 id,
            new_properties:     new_properties
        }
    }

    ///
    /// Retrieves the brush properties that this will set for future elements
    ///
    pub fn brush_properties<'a>(&'a self) -> &BrushProperties {
        &self.new_properties
    }
}

impl VectorElement for BrushPropertiesElement {
    ///
    /// The ID of this vector element
    ///
    fn id(&self) -> ElementId {
        self.id
    }

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    fn to_path(&self, _properties: &VectorProperties) -> Option<Vec<Path>> {
        None
    }

    ///
    /// Renders this vector element
    ///
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, _when: Duration) {
        gc.draw_list(properties.brush.prepare_to_render(&self.new_properties));
    }

    ///
    /// Updates the vector properties for future elements
    ///
    fn update_properties(&self, properties: Arc<VectorProperties>) -> Arc<VectorProperties> {
        let mut properties = (*properties).clone();
        properties.brush_properties = self.new_properties.clone();

        Arc::new(properties)
    }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    ///
    fn motion_transform(&self, _motion: &Motion, _when: Duration) -> Vector {
        Vector::BrushProperties(self.clone())
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
    fn with_adjusted_control_points(&self, _new_positions: Vec<(f32, f32)>) -> Vector {
        Vector::BrushProperties(self.clone())
    }
}

impl Into<Vector> for BrushPropertiesElement {
    #[inline]
    fn into(self) -> Vector {
        Vector::BrushProperties(self)
    }
}
