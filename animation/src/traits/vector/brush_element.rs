use super::*;

use super::super::path::*;
use super::super::edit::*;
use super::super::brush::*;

use std::sync::*;

///
/// Element representing a brush stroke
///
#[derive(Clone)]
pub struct BrushElement {
    /// The ID of this element
    id: ElementId,

    /// The path taken by this brush stroke
    points: Arc<Vec<BrushPoint>>,
}

impl BrushElement {
    ///
    /// Begins a new brush stroke at a particular position
    /// 
    pub fn new(id: ElementId, points: Arc<Vec<BrushPoint>>) -> BrushElement {
        BrushElement {
            id:                 id,
            points:             points
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
    /// The ID of this vector element
    /// 
    fn id(&self) -> ElementId {
        unimplemented!();
    }

    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties) {
        gc.draw_list(properties.brush.render_brush(&properties.brush_properties, &self.points))
    }

    ///
    /// Retrieves the paths for this element, if there are any
    /// 
    fn to_path(&self, properties: &VectorProperties) -> Option<Vec<Path>> {
        Some(vec![Path::from_drawing(properties.brush.render_brush(&properties.brush_properties, &self.points))])
    }
}

impl Into<Vector> for BrushElement {
    #[inline]
    fn into(self) -> Vector {
        Vector::BrushStroke(self)
    }
}
