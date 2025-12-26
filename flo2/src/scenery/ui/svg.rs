use flo_draw::canvas::*;
use flo_svg::*;

///
/// Returns the rendering instructions for an SVG file, with the center of the view box at 0,0
///
pub fn svg(svg: &[u8]) -> Vec<Draw> {
    // Convert to a string
    let svg         = str::from_utf8(svg).unwrap();

    // Generate the main drawing instructions
    let mut drawing = vec![];
    let document    = parse_svg(svg, &mut drawing).unwrap();

    if let Some(((min_x, min_y), (max_x, max_y))) = document.viewbox() {
        // Translate the center of the viewbox to the 0,0 position
        let center_pos  = ((min_x+max_x)/2.0, ((min_y+max_y)/2.0));
        let translation = Transform2D::translate(-center_pos.0, -center_pos.1);

        drawing.splice(0..0, vec![
            Draw::PushState,
            Draw::MultiplyTransform(translation),
        ]);

        drawing.push(Draw::PopState);
    }

    drawing
}

///
/// Returns the rendering instructions for an SVG file, with the center of the view box at 0,0 and scaled to the specified width
///
pub fn svg_with_width(svg: &[u8], width: f64) -> Vec<Draw> {
    // Convert to a string
    let svg         = str::from_utf8(svg).unwrap();

    // Generate the main drawing instructions
    let mut drawing = vec![];
    let document    = parse_svg(svg, &mut drawing).unwrap();

    if let Some(((min_x, min_y), (max_x, max_y))) = document.viewbox() {
        // Translate the center of the viewbox to the 0,0 position
        let center_pos  = ((min_x+max_x)/2.0, ((min_y+max_y)/2.0));
        let translation = Transform2D::translate(-center_pos.0, -center_pos.1);
        let scale       = (width as f32)/(max_x-min_x);
        let scale       = Transform2D::scale(scale, scale);

        drawing.splice(0..0, vec![
            Draw::PushState,
            Draw::MultiplyTransform(translation * scale),
        ]);

        drawing.push(Draw::PopState);
    }

    drawing
}
