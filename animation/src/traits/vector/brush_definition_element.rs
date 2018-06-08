use super::vector::*;
use super::element::*;
use super::properties::*;
use super::super::path::*;
use super::super::edit::*;
use super::super::motion::*;
use super::super::brush_definition::*;
use super::super::brush_drawing_style::*;
use super::super::super::brushes::*;

use canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Element representing selecting a new brush definition
///
#[derive(Clone)]
pub struct BrushDefinitionElement {
    /// The ID of this element
    id: ElementId,

    /// The brush properties to set
    new_definition: BrushDefinition,

    /// The drawing style to use
    drawing_style: BrushDrawingStyle
}

impl BrushDefinitionElement {
    ///
    /// Creates a new brush properties vector element
    /// 
    pub fn new(id: ElementId, new_definition: BrushDefinition, drawing_style: BrushDrawingStyle) -> BrushDefinitionElement {
        BrushDefinitionElement {
            id:             id,
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
    /// The ID of this vector element
    /// 
    fn id(&self) -> ElementId {
        self.id
    }

    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties) {
        gc.draw_list(properties.brush.prepare_to_render(&properties.brush_properties));
    }

    ///
    /// Updates the vector properties for future elements
    /// 
    fn update_properties(&self, properties: Arc<VectorProperties>) -> Arc<VectorProperties> {
        let mut properties = (*properties).clone();
        properties.brush = create_brush_from_definition(&self.new_definition, self.drawing_style);

        Arc::new(properties)
    }

    ///
    /// Retrieves the paths for this element, if there are any
    /// 
    fn to_path(&self, _properties: &VectorProperties) -> Option<Vec<Path>> {
        None
    }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    /// 
    fn motion_transform(&self, _motion: &Motion, _when: Duration) -> Vector {
        Vector::BrushDefinition(self.clone())
    }
}

impl Into<Vector> for BrushDefinitionElement {
    #[inline]
    fn into(self) -> Vector {
        Vector::BrushDefinition(self)
    }
}
