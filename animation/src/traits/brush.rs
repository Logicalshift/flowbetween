use ui::canvas::*;

///
/// Represents a segment of a brush stroke
/// 
#[derive(Clone, Copy)]
pub struct BrushPoint {
    /// Position of this segment
    pub position: (f32, f32),

    /// Pressure (0-1) of this segment
    pub pressure: f32
}

///
/// Trait implemented by things that can draw brush strokes
/// 
pub trait Brush {
    ///
    /// Renders a brush stroke to the specified graphics context
    ///
    fn render_brush(&self, gc: &mut GraphicsContext, points: &Vec<BrushPoint>);
}
