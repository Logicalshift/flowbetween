use super::super::edit::*;
use super::super::brush::*;
use super::super::vector::*;
use super::super::brush_properties::*;
use super::super::brush_definition::*;
use super::super::brush_drawing_style::*;
use super::super::super::brushes::*;

use flo_canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Represents the active properties for a vector layer
///
/// Elements can update the properties, which persist to the next element.
/// This saves some space in that properties don't need to be encoded with
/// each element.
///
#[derive(Clone)]
pub struct VectorProperties {
    /// The active brush
    pub brush: Arc<dyn Brush>,

    /// The properties set for the active brush
    pub brush_properties: BrushProperties,

    /// Transformations to apply to the element during rendering
    pub transformations: Arc<Vec<Transformation>>,

    /// Returns the 
    pub retrieve_attachments: Arc<dyn (Fn(ElementId) -> Vec<Vector>) + Sync+Send>,

    /// Provides an override for how a vector element is rendered
    pub render_vector: Arc<dyn (Fn(&mut dyn GraphicsPrimitives, Vector, Duration, &VectorProperties)) + Sync+Send>
}

impl VectorProperties {
    ///
    /// Creates the default brush properties
    ///
    pub fn default() -> VectorProperties {
        VectorProperties {
            brush:                  Arc::new(InkBrush::new(&InkDefinition::default(), BrushDrawingStyle::Draw)),
            brush_properties:       BrushProperties::new(),
            transformations:        Arc::new(vec![]),
            retrieve_attachments:   Arc::new(|_| vec![]),
            render_vector:          Arc::new(|gc, vector, when, properties| vector.render(gc, properties, when))
        }
    }

    ///
    /// Prepares the context to render with these properties
    ///
    pub fn prepare_to_render(&self, gc: &mut dyn GraphicsPrimitives) {
        gc.draw_list(self.brush.prepare_to_render(&self.brush_properties));
    }

    ///
    /// Renders the specified element with these properties
    ///
    pub fn render(&self, gc: &mut dyn GraphicsPrimitives, element: Vector, when: Duration) {
        // Render this element
        (self.render_vector)(gc, element, when, self);
    }
}
