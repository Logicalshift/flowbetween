use super::*;
use super::super::path::*;
use super::super::brush_definition::*;
use super::super::brush_drawing_style::*;
use super::super::super::brushes::*;

///
/// Element representing selecting a new brush definition
///
#[derive(Clone)]
pub struct BrushDefinitionElement {
    /// The brush properties to set
    new_definition: BrushDefinition,

    /// The drawing style to use
    drawing_style: BrushDrawingStyle
}

impl BrushDefinitionElement {
    ///
    /// Creates a new brush properties vector element
    /// 
    pub fn new(new_definition: BrushDefinition, drawing_style: BrushDrawingStyle) -> BrushDefinitionElement {
        BrushDefinitionElement {
            new_definition: new_definition,
            drawing_style:  drawing_style
        }
    }

    ///
    /// Retrieves the brush definition for this element
    ///
    pub fn definition<'a>(&'a self) -> &'a BrushDefinition {
        &self.new_definition
    }

    ///
    /// Retrieves the drawing style for this element
    ///
    pub fn drawing_style(&self) -> BrushDrawingStyle {
        self.drawing_style
    }
}

impl VectorElement for BrushDefinitionElement {
    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties) {
        gc.draw_list(properties.brush.prepare_to_render(&properties.brush_properties));
    }

    ///
    /// Updates the vector properties for future elements
    /// 
    fn update_properties(&self, properties: &mut VectorProperties) {
        properties.brush = create_brush_from_definition(&self.new_definition, self.drawing_style);
    }

    ///
    /// Retrieves the paths for this element, if there are any
    /// 
    fn to_path(&self, _properties: &VectorProperties) -> Option<Vec<Path>> {
        None
    }

}

impl Into<Vector> for BrushDefinitionElement {
    #[inline]
    fn into(self) -> Vector {
        Vector::BrushDefinition(self)
    }
}
