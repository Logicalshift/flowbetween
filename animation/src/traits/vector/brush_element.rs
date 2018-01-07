use super::element::*;
use super::properties::*;

use super::super::brush::*;
use super::super::brush_properties::*;

use canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Element representing a brush stroke
///
#[derive(Clone)]
pub struct BrushElement {
    /// When this element is drawn relative to the start of the frame
    appearance_time: Duration,

    /// The path taken by this brush stroke
    points: Vec<BrushPoint>,
}

impl BrushElement {
    ///
    /// Begins a new brush stroke at a particular position
    /// 
    pub fn new(brush: &Arc<Brush>, appearance_time: Duration, start_pos: BrushPoint, properties: &BrushProperties) -> BrushElement {
        BrushElement {
            appearance_time:    appearance_time,
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
    /// Updates the appearance time of this item
    /// 
    pub fn set_appearance_time(&mut self, new_time: Duration) {
        self.appearance_time = new_time;
    }

    ///
    /// Retrieves the points in this brush element
    /// 
    pub fn points<'a>(&'a self) -> &'a Vec<BrushPoint> {
        &self.points
    }
}

impl VectorElement for BrushElement {
    fn appearance_time(&self) -> Duration {
        self.appearance_time
    }

    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties) {
        // TODO: find a way to only prepare to render once per brush
        properties.brush.prepare_to_render(gc, &properties.brush_properties);
        properties.brush.render_brush(gc, &properties.brush_properties, &self.points)
    }
}