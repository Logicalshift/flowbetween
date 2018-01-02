use super::super::traits::*;

use curves::*;
use curves::bezier;
use canvas::*;

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
    fn prepare_to_render(&self, gc: &mut GraphicsPrimitives, properties: &BrushProperties) {
        gc.blend_mode(BlendMode::SourceOver);
        gc.stroke_color(properties.color);
    }

    ///
    /// Retrieves the definition for this brush
    /// 
    fn to_definition(&self) -> (BrushDefinition, BrushDrawingStyle) {
        (BrushDefinition::Simple, BrushDrawingStyle::Draw)
    }

    fn render_brush(&self, gc: &mut GraphicsPrimitives, points: &Vec<BrushPoint>) {
        // Nothing to draw if there are no points in the brush stroke (or only one point)
        if points.len() <= 1 {
            return;
        }

        // Map to coordinates
        let coords: Vec<Coord2> = points.iter().map(|point| Coord2(point.position.0 as f64, point.position.1 as f64)).collect();

        // Pick points that are at least a certain distance apart to use for the fitting algorithm
        let mut distant_coords  = vec![];
        let mut last_point      = coords[0];

        distant_coords.push(last_point);
        for x in 1..coords.len() {
            if last_point.distance_to(&coords[x]) >= 2.0 {
                last_point = coords[x];
                distant_coords.push(last_point);
            }
        }

        // Fit these points to a curve
        let curve = bezier::Curve::fit_from_points(&distant_coords, 2.0);
        
        // Draw a simple line for this brush
        if let Some(curve) = curve {
            gc.new_path();
            
            let Coord2(x, y) = curve[0].start_point();
            gc.move_to(x as f32, y as f32);
            for curve_section in curve.iter() {
                gc_draw_bezier(gc, curve_section);
            }

            gc.stroke();
        }
    }
}