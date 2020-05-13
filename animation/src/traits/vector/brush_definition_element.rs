use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::vector_element::*;
use super::path_conversion_options::*;
use super::super::path::*;
use super::super::edit::*;
use super::super::brush_definition::*;
use super::super::brush_drawing_style::*;
use super::super::super::brushes::*;

use flo_canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Element representing selecting a new brush definition
///
#[derive(Clone, Debug)]
pub struct BrushDefinitionElement {
    /// The ID of this element
    id: ElementId,

    /// The brush properties to set
    new_definition: BrushDefinition,

    /// The drawing style to use
    drawing_style: BrushDrawingStyle
}

impl Default for BrushDefinitionElement {
    fn default() -> BrushDefinitionElement {
        BrushDefinitionElement::new(ElementId::Unassigned, BrushDefinition::Simple, BrushDrawingStyle::Draw)
    }
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
    /// Modifies this element to have a new ID
    ///
    fn set_id(&mut self, new_id: ElementId) {
        self.id = new_id
    }

    ///
    /// Renders this vector element
    ///
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, _when: Duration) {
        gc.draw_list(properties.brush.prepare_to_render(&properties.brush_properties));
    }

    ///
    /// Updates the vector properties for future elements
    ///
    fn update_properties(&self, properties: Arc<VectorProperties>, _when: Duration) -> Arc<VectorProperties> {
        let mut properties = (*properties).clone();
        properties.brush = create_brush_from_definition(&self.new_definition, self.drawing_style);

        Arc::new(properties)
    }

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    fn to_path(&self, _properties: &VectorProperties, _options: PathConversion) -> Option<Vec<Path>> {
        None
    }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self, _properties: &VectorProperties) -> Vec<ControlPoint> {
        vec![]
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    fn with_adjusted_control_points(&self, _new_positions: Vec<(f32, f32)>, _properties: &VectorProperties) -> Vector {
        Vector::BrushDefinition(self.clone())
    }
}

impl Into<Vector> for BrushDefinitionElement {
    #[inline]
    fn into(self) -> Vector {
        Vector::BrushDefinition(self)
    }
}
