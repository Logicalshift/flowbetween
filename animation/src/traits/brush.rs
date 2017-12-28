use ui::*;
use ui::canvas::*;

///
/// Represents a segment of a brush stroke
/// 
#[derive(Clone, Copy, PartialEq)]
pub struct BrushPoint {
    /// Position of this segment
    pub position: (f32, f32),

    /// Pressure (0-1) of this segment
    pub pressure: f32
}

///
/// Trait implemented by things that can draw brush strokes
/// 
pub trait Brush : Send+Sync {
    ///
    /// One or more brush strokes of this type are about to be rendered.
    /// This brush should set up the graphics context appropriately.
    /// 
    fn prepare_to_render(&self, gc: &mut GraphicsPrimitives);

    ///
    /// Renders a brush stroke to the specified graphics context
    ///
    fn render_brush(&self, gc: &mut GraphicsPrimitives, points: &Vec<BrushPoint>);
}

impl From<(f32, f32)> for BrushPoint {
    fn from(pos: (f32, f32)) -> BrushPoint {
        BrushPoint {
            position: pos,
            pressure: 1.0
        }
    }
}

impl<'a> From<&'a Painting> for BrushPoint {
    fn from(painting: &'a Painting) -> BrushPoint {
        BrushPoint {
            position: painting.location,
            pressure: painting.pressure
        }
    }
}