use crate::traits::edit::*;
use crate::traits::path::*;
use crate::traits::brush::*;
use crate::traits::vector::*;

use flo_curves::*;
use flo_curves::arc::*;
use flo_curves::bezier::path::*;
use flo_canvas::*;

use std::f64;
use std::sync::*;
use std::time::{Duration};

///
/// Shapes that can be represented by the shape element
///
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
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
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ShapeElement {
    /// The ID of this element
    id: ElementId,

    /// The width of the brush stroke to generate for this shape
    width: f64,

    /// The shape of this element
    shape: Shape
}

impl ShapeElement {
    pub fn new(element_id: ElementId, width: f64, shape: Shape) -> ShapeElement {
        ShapeElement {
            id:     element_id,
            width:  width,
            shape:  shape
        }
    }

    ///
    /// Creates a new circle shape
    ///
    pub fn circle(element_id: ElementId, center: (f64, f64), point: (f64, f64)) -> ShapeElement {
        ShapeElement {
            id:     element_id,
            width:  0.5,
            shape:  Shape::Circle { center: center, point: point }
        }
    }

    ///
    /// Creates a new rectangle shape
    ///
    pub fn rectangle(element_id: ElementId, center: (f64, f64), point: (f64, f64)) -> ShapeElement {
        ShapeElement {
            id:     element_id,
            width:  0.5,
            shape:  Shape::Rectangle { center: center, point: point }
        }
    }

    ///
    /// Creates a new polygon shape
    ///
    pub fn polygon(element_id: ElementId, center: (f64, f64), point: (f64, f64), sides: usize) -> ShapeElement {
        ShapeElement {
            id:     element_id,
            width:  0.5,
            shape:  Shape::Polygon { sides: sides, center: center, point: point }
        }
    }

    ///
    /// Returns the brush points to use when drawing this shape element
    ///
    pub fn brush_points(&self) -> Vec<BrushPoint> {
        match self.shape {
            Shape::Circle { center, point }         => self.brush_points_circle(center, point),
            Shape::Rectangle { center, point }      => self.brush_points_rectangle(center, point),
            Shape::Polygon { sides, center, point } => self.brush_points_polygon(sides, center, point)
        }
    }

    ///
    /// Retrieves the shape represented by this element
    ///
    pub fn shape(&self) -> Shape {
        self.shape
    }

    ///
    /// Retrieves the line width that will be used with the brush applied to this element
    ///
    pub fn width(&self) -> f64 {
        self.width
    }

    fn brush_points_circle(&self, center: (f64, f64), point: (f64, f64)) -> Vec<BrushPoint> {
        // Create a new unit circle path
        let circle      = Circle::new(PathPoint::new(0.0, 0.0), 0.5).to_path::<Path>();

        // Convert to an ellipse by transforming it
        let (cx, cy)    = center;
        let (px, py)    = point;
        let width       = (px-cx).abs()*2.0;
        let height      = (py-cy).abs()*2.0;
        let scale       = Transformation::Scale(width, height, (0.0, 0.0));
        let translate   = Transformation::Translate(cx, cy);

        let circle      = scale.transform_path(&circle);
        let circle      = translate.transform_path(&circle);

        // Convert the path to brush components
        let mut last_point      = PathPoint::new(0.0, 0.0);
        let mut brush_points    = vec![];
        for component in circle.elements() {
            match component {
                PathComponent::Move(point) => { last_point = point },
                PathComponent::Line(point) => {
                    brush_points.push(BrushPoint::from_path_component(&last_point, &component, self.width));
                    last_point = point;
                }

                PathComponent::Bezier(point, _cp1, _cp2) => {
                    brush_points.push(BrushPoint::from_path_component(&last_point, &component, self.width));
                    last_point = point;
                }

                PathComponent::Close => { }
            }
        }

        brush_points
    }

    fn brush_points_rectangle(&self, center: (f64, f64), point: (f64, f64)) -> Vec<BrushPoint> {
        let mut brush_points = vec![];

        // Draw a rectangle centered around the specified point
        let (cx, cy) = center;
        let (px, py) = point;
        let (dx, dy) = ((cx-px).abs(), (cy-py).abs());

        brush_points.push(BrushPoint::from_line((cx-dx, cy-dy), (cx+dx, cy-dy), self.width));
        brush_points.push(BrushPoint::from_line((cx+dx, cy-dy), (cx+dx, cy+dy), self.width));
        brush_points.push(BrushPoint::from_line((cx+dx, cy+dy), (cx-dx, cy+dy), self.width));
        brush_points.push(BrushPoint::from_line((cx-dx, cy+dy), (cx-dx, cy-dy), self.width));

        brush_points
    }

    fn brush_points_polygon(&self, sides: usize, center: (f64, f64), point: (f64, f64)) -> Vec<BrushPoint> {
        let mut brush_points = vec![];

        let (cx, cy)        = center;
        let (px, py)        = point;
        let (dx, dy)        = ((cx-px).abs(), (cy-py).abs());
        let radius          = ((dx*dx)+(dy*dy)).sqrt();
        let initial_angle   = f64::atan2(dy, dx);
        let angle_per_side  = (2.0*f64::consts::PI) / (sides as f64);

        for side in 0..sides {
            let start_angle = (side as f64) * angle_per_side + initial_angle;
            let end_angle   = start_angle + angle_per_side;

            brush_points.push(BrushPoint::from_line((start_angle.sin()*radius + cx, start_angle.cos()*radius + cy), (end_angle.sin()*radius + cx, end_angle.cos()*radius + cy), self.width));
        }

        brush_points
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
        // Convert the brush stroke to the simplest path we can
        let points          = self.brush_points();
        let simplest_path   = vec![Path::from_drawing(properties.brush.render_brush(&properties.brush_properties, &points, Arc::clone(&properties.transformations)))];

        // Final result depends on the options that are set
        match options {
            PathConversion::Fastest                 => Some(simplest_path),
            PathConversion::RemoveInteriorPoints    => {
                let path = path_remove_interior_points(&simplest_path, 0.01);
                let path = Path::from_paths(&path);
                Some(vec![path])
            }
        }
    }

    ///
    /// Renders this vector element
    ///
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, _when: Duration) {
        let points = self.brush_points();

        gc.draw_list(properties.brush.render_brush(&properties.brush_properties, &points, Arc::clone(&properties.transformations)))
    }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self, properties: &VectorProperties) -> Vec<ControlPoint> {
        let before_transforms = match self.shape {
            Shape::Circle { center, point }                 => vec![ControlPoint::BezierPoint(center.0, center.1), ControlPoint::BezierPoint(point.0, point.1)],
            Shape::Rectangle { center, point }              => vec![ControlPoint::BezierPoint(center.0, center.1), ControlPoint::BezierPoint(point.0, point.1)],
            Shape::Polygon { sides: _sides, center, point } => vec![ControlPoint::BezierPoint(center.0, center.1), ControlPoint::BezierPoint(point.0, point.1)],
        };

        before_transforms.into_iter().map(|point| point.apply_transformations(&*properties.transformations)).collect()
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    fn with_adjusted_control_points(&self, new_positions: Vec<(f32, f32)>, properties: &VectorProperties) -> Vector {
        if new_positions.len() != 2 {
            // Must be two adjusted control points
            Vector::Shape(self.clone())
        } else {
            // Invert any transforms
            let transforms = properties.transformations.iter()
                .flat_map(|transform| transform.invert())
                .collect::<Vec<_>>();

            // Update the control points by transforming them
            let control_points = new_positions.into_iter()
                .map(|cp| Coord2(cp.0 as f64, cp.1 as f64))
                .map(|cp| transforms.iter().fold(cp, |cp, transform| transform.transform_point(&cp)))
                .map(|cp| (cp.x(), cp.y()))
                .collect::<Vec<_>>();

            // Create a new shape with the new control points
            let new_shape = match self.shape {
                Shape::Circle { center: _center, point: _point}             => Self::circle(self.id, control_points[0], control_points[1]),
                Shape::Rectangle { center: _center, point: _point}          => Self::rectangle(self.id, control_points[0], control_points[1]),
                Shape::Polygon { sides, center: _center, point: _point }    => Self::polygon(self.id, control_points[0], control_points[1], sides)
            };

            Vector::Shape(new_shape)
        }
    }
}
