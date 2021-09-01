use flo_canvas::*;
use flo_canvas_animation::*;

#[test]
pub fn simple_circle_path() {
    let mut drawing         = vec![];
    let mut drawing_to_path = LayerDrawingToPaths::new();

    drawing.circle(100.0, 200.0, 50.0);
    drawing.stroke_color(Color::Rgba(0.1, 0.2, 0.3, 0.4));
    drawing.fill_color(Color::Rgba(0.3, 0.4, 0.5, 0.6));
    drawing.fill();

    let paths               = drawing_to_path.draw(drawing).collect::<Vec<_>>();

    assert!(paths.len() == 1);
    assert!(paths[0].appearance_time == 0.0);
    assert!(paths[0].attributes == AnimationPathAttribute::Fill(Color::Rgba(0.3, 0.4, 0.5, 0.6), WindingRule::EvenOdd));

    // 6 ops: 1 move, 4 bezier curves, 1 close
    assert!(paths[0].path.len() == 6);
}
