use super::*;

use super::super::brush::*;

use std::sync::*;

///
/// Element representing a brush stroke
///
#[derive(Clone)]
pub struct BrushElement {
    /// The path taken by this brush stroke
    points: Arc<Vec<BrushPoint>>,
}

impl BrushElement {
    ///
    /// Begins a new brush stroke at a particular position
    /// 
    pub fn new(points: Arc<Vec<BrushPoint>>) -> BrushElement {
        BrushElement {
            points:             points,
        }
    }

    ///
    /// Retrieves the points in this brush element
    /// 
    pub fn points(&self) -> Arc<Vec<BrushPoint>> {
        Arc::clone(&self.points)
    }
}

impl VectorElement for BrushElement {
    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties) {
        properties.brush.render_brush(gc, &properties.brush_properties, &self.points)
    }

    ///
    /// Retrieves the paths for this element, if there are any
    /// 
    fn to_path(&self, _properties: &VectorProperties) -> Option<Vec<Path>> {
        None
    }
}

impl Into<Vector> for BrushElement {
    #[inline]
    fn into(self) -> Vector {
        Vector::BrushStroke(self)
    }
}
