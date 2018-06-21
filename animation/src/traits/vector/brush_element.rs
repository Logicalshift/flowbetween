use super::vector::*;
use super::element::*;
use super::properties::*;
use super::super::path::*;
use super::super::edit::*;
use super::super::brush::*;
use super::super::motion::*;

use canvas::*;

use std::sync::*;
use std::time::Duration;

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

    ///
    /// Moves this brush stroke so that it fits within a particular bounding box
    /// (when rendered with a particular set of properties)
    /// 
    pub fn move_to(&mut self, new_bounds: Rect, properties: &VectorProperties) {
        // Scale using the existing bounds
        let existing_bounds = self.to_path(properties)
            .map(|paths| paths.into_iter()
                .map(|path| Rect::from(&path))
                .fold(Rect::empty(), |a, b| a.union(b)))
            .unwrap_or(Rect::empty());

        let (current_w, current_h)  = (existing_bounds.x2-existing_bounds.x1, existing_bounds.y2-existing_bounds.y1);
        let (new_w, new_h)          = (new_bounds.x2-new_bounds.x1, new_bounds.y2-new_bounds.y1);
        let (scale_x, scale_y)      = (new_w/current_w, new_h/current_h);

        // Functions to transform the points in this brush stroke
        let transform       = |(x, y)| {
            ((x - existing_bounds.x1)*scale_x + new_bounds.x1,
             (y - existing_bounds.y1)*scale_y + new_bounds.y1)
        };

        let transform_point = |point: &BrushPoint| {
            BrushPoint {
                position:   transform(point.position),
                cp1:        transform(point.cp1),
                cp2:        transform(point.cp2),
                width:      point.width
            }
        };

        // Perform the transformation itself
        let new_points      = self.points.iter()
            .map(|old_point| transform_point(old_point))
            .collect();
        self.points = Arc::new(new_points);
    }
}

impl VectorElement for BrushElement {
    ///
    /// The ID of this vector element
    /// 
    fn id(&self) -> ElementId {
        self.id
    }

    ///
    /// Renders this vector element
    /// 
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties) {
        gc.draw_list(properties.brush.render_brush(&properties.brush_properties, &self.points))
    }

    ///
    /// Retrieves the paths for this element, if there are any
    /// 
    fn to_path(&self, properties: &VectorProperties) -> Option<Vec<Path>> {
        Some(vec![Path::from_drawing(properties.brush.render_brush(&properties.brush_properties, &self.points))])
    }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    /// 
    fn motion_transform(&self, motion: &Motion, when: Duration) -> Vector {
        let transformed_points = motion.transform_points(when, self.points.iter()).collect();

        Vector::BrushStroke(BrushElement {
            id:     self.id,
            points: Arc::new(transformed_points)
        })
    }
}

impl Into<Vector> for BrushElement {
    #[inline]
    fn into(self) -> Vector {
        Vector::BrushStroke(self)
    }
}
