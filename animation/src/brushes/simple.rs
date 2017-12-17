use super::super::traits::*;
use ui::canvas::*;

use curves::*;
use curves::bezier;

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

        // Fit a curve to the points
        let coords  = points.iter().map(|point| Coord2(point.position.0, point.position.1)).collect();
        let curve   = bezier::Curve::fit_from_points(&coords, 2.0);
        
        // Draw a simple line for this brush
        if let Some(curve) = curve {
            gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
            gc.new_path();
            
            let Coord2(x, y) = curve[0].start_point();
            gc.move_to(x, y);
            for curve_section in curve.iter() {
                gc_draw_bezier(gc, curve_section);
            }

            gc.stroke();        
        }
    }
}