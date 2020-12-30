use crate::traits::edit::*;
use crate::traits::path::*;
use crate::traits::brush::*;
use crate::traits::vector::*;

use flo_canvas::*;

use std::time::{Duration};

///
/// Shapes that can be represented by the shape element
///
pub enum Shape {
    /// A circle
    Circle { center: (f64, f64), point: (f64, f64) },

    /// A rectangle (always axis-aligned)
    Rectangle { center: (f64, f64), point: (f64, f64) },

    /// A polygon
    Polygon { sides: usize, center: (f64, f64), point: (f64, f64) },

}

///
/// Element that describes a regular shape
///
pub struct ShapeElement {
    /// The ID of this element
    id: ElementId,

    /// The shape of this element
    shape: Shape
}

impl ShapeElement {
    ///
    /// Creates a new circle shape
    ///
    pub fn circle(element_id: ElementId, center: (f64, f64), point: (f64, f64)) -> ShapeElement {
        ShapeElement {
            id:     element_id,
            shape:  Shape::Circle { center: center, point: point }
        }
    }

    ///
    /// Creates a new rectangle shape
    ///
    pub fn rectangle(element_id: ElementId, center: (f64, f64), point: (f64, f64)) -> ShapeElement {
        ShapeElement {
            id:     element_id,
            shape:  Shape::Rectangle { center: center, point: point }
        }
    }

    ///
    /// Creates a new polygon shape
    ///
    pub fn polygon(element_id: ElementId, center: (f64, f64), point: (f64, f64), sides: usize) -> ShapeElement {
        ShapeElement {
            id:     element_id,
            shape:  Shape::Polygon { sides: sides, center: center, point: point }
        }
    }

    ///
    /// Returns the brush points to use when drawing this shape element
    ///
    pub fn brush_points(&self) -> Vec<BrushPoint> {
        unimplemented!()
    }
}

impl VectorElement for ShapeElement {
    ///
    /// The ID of this element
    ///
    fn id(&self) -> ElementId { self.id }

    ///
    /// Modifies this element to have a new ID
    ///
    fn set_id(&mut self, new_id: ElementId) { self.id = new_id; }

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    fn to_path(&self, properties: &VectorProperties, options: PathConversion) -> Option<Vec<Path>> {
        // TODO
        unimplemented!()
    }

    ///
    /// Renders this vector element
    ///
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, when: Duration) {
        // TODO
    }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self, properties: &VectorProperties) -> Vec<ControlPoint> {
        // TODO
        unimplemented!()
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>, properties: &VectorProperties) -> Vector {
        // TODO
        unimplemented!()
    }
}