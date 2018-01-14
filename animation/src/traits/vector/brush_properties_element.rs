use super::*;
use super::super::brush_properties::*;

///
/// Element representing selecting some new brush properties
///
#[derive(Clone)]
pub struct BrushPropertiesElement {
    /// The brush properties to set
    new_properties: BrushProperties
}

impl BrushPropertiesElement {
    ///
    /// Creates a new brush properties vector element
    /// 
    pub fn new(new_properties: BrushProperties) -> BrushPropertiesElement {
        BrushPropertiesElement {
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

impl Into<Vector> for BrushPropertiesElement {
    #[inline]
    fn into(self) -> Vector {
        Vector::BrushProperties(self)
    }
}
