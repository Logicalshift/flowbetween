use super::*;
use super::super::path::*;
use super::super::edit::*;
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
    /// The ID of this vector element
    /// 
    fn id(&self) -> ElementId {
        unimplemented!()
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
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties) {
        gc.draw_list(properties.brush.prepare_to_render(&self.new_properties));
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
