use super::super::vector::*;
use super::super::brush::*;
use super::super::brush_properties::*;
use super::super::brush_definition::*;
use super::super::brush_drawing_style::*;
use super::super::super::brushes::*;

use canvas::*;

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

    /// Transformation to apply before rendering an element with these properties
    pub transform: Arc<dyn (Fn(Vector, Duration) -> Vector) + Sync+Send>
}

impl VectorProperties {
    ///
    /// Creates the default brush properties
    /// 
    pub fn default() -> VectorProperties {
        VectorProperties {
            brush:              Arc::new(InkBrush::new(&InkDefinition::default(), BrushDrawingStyle::Draw)),
            brush_properties:   BrushProperties::new(),
            transform:          Arc::new(|vector, _when| vector)
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
        // Apply the transformation, if there is one
        let element = (self.transform)(element, when);

        // Render this element
        element.render(gc, self);
    }
}
