use super::element::*;
use super::properties::*;

use super::super::brush::*;

use canvas::*;

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
    pub fn points<'a>(&'a self) -> &'a Vec<BrushPoint> {
        &*self.points
    }
}

impl VectorElement for BrushElement {
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties) {
        properties.brush.render_brush(gc, &properties.brush_properties, &self.points)
    }
}