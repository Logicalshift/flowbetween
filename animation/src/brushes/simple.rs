use super::super::traits::*;
use ui::canvas::*;

///
/// Simple brush, which renders a brush stroke as a straight series of line segments
/// 
pub struct SimpleBrush {

}

impl SimpleBrush {
    pub fn new() -> SimpleBrush {
        SimpleBrush { }
    }
}

impl Brush for SimpleBrush {
    fn render_brush(&self, gc: &mut GraphicsPrimitives, points: &Vec<BrushPoint>) {
        // Nothing to draw if there are no points in the brush stroke (or only one point)
        if points.len() <= 1 {
            return;
        }
        
        // Draw a simple line for this brush
        gc.new_path();
        
        let (x, y) = points[0].position;
        gc.move_to(x, y);
        for point in points.iter().skip(1) {
            let (x, y) = point.position;
            gc.line_to(x, y);
        }

        gc.stroke();        
    }
}