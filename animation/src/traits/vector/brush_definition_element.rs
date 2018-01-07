use super::element::*;
use super::properties::*;
use super::super::brush_definition::*;
use super::super::brush_drawing_style::*;
use super::super::super::brushes::*;

use canvas::*;

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
}

impl VectorElement for BrushDefinitionElement {
    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties) {
        properties.brush.prepare_to_render(gc, &properties.brush_properties);
    }

    ///
    /// Updates the vector properties for future elements
    /// 
    fn update_properties(&self, properties: &mut VectorProperties) {
        properties.brush = create_brush_from_definition(&self.new_definition, self.drawing_style);
    }
}