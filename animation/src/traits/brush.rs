use super::path::*;
use super::vector::*;
use super::raw_point::*;
use super::combine_result::*;
use super::brush_properties::*;
use super::brush_definition::*;
use super::brush_drawing_style::*;
use super::vector::transformation::*;

use flo_canvas::*;

use std::iter;
use std::sync::*;

///
/// Represents a segment of a brush stroke
///
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct BrushPoint {
    /// Position of this segment
    pub position: (f32, f32),

    /// First control point for this segment
    pub cp1: (f32, f32),

    /// Second control point for this segment
    pub cp2: (f32, f32),

    /// Width of this segment
    pub width: f32
}

impl BrushPoint {
    ///
    /// Creates a brush point from a path component
    ///
    pub fn from_path_component(last_point: &PathPoint, component: &PathComponent, width: f64) -> BrushPoint {
        use self::PathComponent::*;

        let (x1, y1) = last_point.position;

        match component {
            // Move or Line are both treated the same here
            Move(point) | Line(point) => {
                // Generate linear control points for this line
                let (x2, y2)        = point.position;
                let (cpx1, cpy1)    = ((x2-x1)*0.33333 + x1, (y2-y1)*0.33333);
                let (cpx2, cpy2)    = ((x2-x1)*0.66666 + x1, (y2-y1)*0.66666);

                // Create the brush pooint
                BrushPoint {
                    position:   (x2 as f32, y2 as f32),
                    cp1:        (cpx1 as f32, cpy1 as f32),
                    cp2:        (cpx2 as f32, cpy2 as f32),
                    width:      width as f32
                }
            },

            Bezier(target, cp1, cp2) => {
                // Simple brush point
                let (x2, y2)        = target.position;
                let (cpx1, cpy1)    = cp1.position;
                let (cpx2, cpy2)    = cp2.position;

                BrushPoint {
                    position:   (x2 as f32, y2 as f32),
                    cp1:        (cpx1 as f32, cpy1 as f32),
                    cp2:        (cpx2 as f32, cpy2 as f32),
                    width:      width as f32
                }
            },

            // Close is treated as a single point (ideally the caller should convert this to a line)
            Close => { 
                BrushPoint {
                    position:   (x1 as f32, y1 as f32),
                    cp1:        (x1 as f32, y1 as f32),
                    cp2:        (x1 as f32, y1 as f32),
                    width:      width as f32
                }
            }
        }
    }

    ///
    /// Creates a new brush point representing a line
    ///
    /// The 'from' point is represented by the last brush point
    ///
    pub fn from_line(from: (f64, f64), to: (f64, f64), width: f64) -> BrushPoint {
        // Generate linear control points for this line
        let (x1, y1)        = from;
        let (x2, y2)        = to;
        let (cpx1, cpy1)    = ((x2-x1)*0.33333 + x1, (y2-y1)*0.33333 + y1);
        let (cpx2, cpy2)    = ((x2-x1)*0.66666 + x1, (y2-y1)*0.66666 + y1);

        // Create the brush pooint
        BrushPoint {
            position:   (x2 as f32, y2 as f32),
            cp1:        (cpx1 as f32, cpy1 as f32),
            cp2:        (cpx2 as f32, cpy2 as f32),
            width:      width as f32
        }
    }
}

///
/// Trait implemented by things that can draw brush strokes
///
pub trait Brush : Send+Sync {
    ///
    /// Returns the brush points for rendering given a particular set of raw points
    ///
    fn brush_points_for_raw_points(&self, raw_points: &[RawPoint]) -> Vec<BrushPoint>;

    ///
    /// One or more brush strokes of this type are about to be rendered.
    /// This brush should set up the graphics context appropriately.
    ///
    fn prepare_to_render<'a>(&'a self, properties: &'a BrushProperties) -> Box<dyn 'a+Iterator<Item=Draw>>;

    ///
    /// Renders a brush stroke to a set of drawing commands
    ///
    fn render_brush<'a>(&'a self, properties: &'a BrushProperties, points: &'a Vec<BrushPoint>, transform: Arc<Vec<Transformation>>) -> Box<dyn 'a+Iterator<Item=Draw>>;

    ///
    /// Renders a path using this brush's style
    ///
    fn render_path<'a>(&'a self, _properties: &'a BrushProperties, path: &'a Path) -> Box<dyn 'a+Iterator<Item=Draw>> {
        Box::new(iter::once(Draw::Path(PathOp::NewPath))
            .chain(path.to_drawing())
            .chain(iter::once(Draw::Fill)))
    }

    ///
    /// Retrieves the definition for this brush
    ///
    fn to_definition(&self) -> (BrushDefinition, BrushDrawingStyle);

    ///
    /// Retrieves just the drawing style for this brush
    ///
    fn drawing_style(&self) -> BrushDrawingStyle {
        self.to_definition().1
    }

    ///
    /// Attempts to combine this brush stroke with the specified vector element. Returns the combined element if successful
    ///
    /// The combined element is the combination so far. The next element is an element that is found underneath this element that
    /// we're trying to combine with. The result indicates if a new element was generated, and if it was what the newly combined
    /// element was (this should be used with future calls if more elements can be combined in). It can also indicate that an
    /// overlapping element means that elements lower down can't be combined.
    ///
    fn combine_with(&self, _combined_element: &Vector, _combined_element_properties: &VectorProperties, _next_element: &Vector, _next_element_properties: &VectorProperties) -> CombineResult {
        CombineResult::UnableToCombineFurther
    }
}
