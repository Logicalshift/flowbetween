use super::element::*;
use super::properties::*;
use super::super::brush_properties::*;

use canvas::*;

use std::time::Duration;

///
/// Element representing selecting some new brush properties
///
#[derive(Clone)]
pub struct BrushPropertiesElement {
    /// The time when this properties element is applied
    appearance_time: Duration,

    /// The brush properties to set
    new_properties: BrushProperties
}

impl BrushPropertiesElement {
    ///
    /// Creates a new brush properties vector element
    /// 
    pub fn new(appearance_time: Duration, new_properties: BrushProperties) -> BrushPropertiesElement {
        BrushPropertiesElement {
            appearance_time:    appearance_time,
            new_properties:     new_properties
        }
    }
}

impl VectorElement for BrushPropertiesElement {
    fn appearance_time(&self) -> Duration {
        self.appearance_time
    }

    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties) {
        properties.brush.prepare_to_render(gc, &self.new_properties);
    }

    ///
    /// Updates the vector properties for future elements
    /// 
    fn update_properties(&self, properties: &mut VectorProperties) {
        properties.brush_properties = self.new_properties.clone();
    }
}