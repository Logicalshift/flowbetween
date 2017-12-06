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
