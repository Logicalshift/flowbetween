use super::element::*;
use super::properties::*;

use super::super::brush::*;

use canvas::*;

///
/// Element representing a brush stroke
///
#[derive(Clone)]
pub struct BrushElement {
    /// The path taken by this brush stroke
    points: Vec<BrushPoint>,
}

impl BrushElement {
    ///
    /// Begins a new brush stroke at a particular position
    /// 
    pub fn new(start_pos: BrushPoint) -> BrushElement {
        BrushElement {
            points:             vec![start_pos],
        }
    }

    ///
    /// Adds a new brush point to this item
    /// 
    pub fn add_point(&mut self, point: BrushPoint) {
        self.points.push(point);
    }

    ///
    /// Retrieves the points in this brush element
    /// 
    pub fn points<'a>(&'a self) -> &'a Vec<BrushPoint> {
        &self.points
    }
}

impl VectorElement for BrushElement {
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties) {
        properties.brush.render_brush(gc, &properties.brush_properties, &self.points)
    }
}