use flo_canvas::*;
use flo_curves::*;
use flo_curves::bezier::path::*;
use flo_canvas_animation::*;

use std::time::{Duration};

#[test]
pub fn simple_circle_path() {
    let mut drawing         = vec![];
    let mut drawing_to_path = LayerDrawingToPaths::new();

    drawing.circle(100.0, 200.0, 50.0);
    drawing.stroke_color(Color::Rgba(0.1, 0.2, 0.3, 0.4));
    drawing.fill_color(Color::Rgba(0.3, 0.4, 0.5, 0.6));
    drawing.fill();

    let paths               = drawing_to_path.draw(drawing).collect::<Vec<_>>();

    assert!(paths.len()                 == 1);
    assert!(paths[0].appearance_time    == Duration::from_millis(0));
    assert!(paths[0].attributes         == AnimationPathAttribute::Fill(Color::Rgba(0.3, 0.4, 0.5, 0.6), WindingRule::EvenOdd));

    // 6 ops: 1 move, 4 bezier curves, 1 close
    assert!(paths[0].path.len()         == 6);
}

#[test]
pub fn components_from_simple_circle() {
    let mut drawing         = vec![];
    let mut drawing_to_path = LayerDrawingToPaths::new();

    drawing.circle(100.0, 200.0, 50.0);
    drawing.stroke_color(Color::Rgba(0.1, 0.2, 0.3, 0.4));
    drawing.fill_color(Color::Rgba(0.3, 0.4, 0.5, 0.6));
    drawing.fill();

    let paths               = drawing_to_path.draw(drawing).collect::<Vec<_>>();

    assert!(paths.len() == 1);

    let circle = PathComponent::components_from_path(&paths[0]);
    
    assert!(circle.len()                == 1);
    assert!(circle[0].points().count()  == 4);
}

#[test]
pub fn simple_circle_bounds() {
    let mut drawing         = vec![];
    let mut drawing_to_path = LayerDrawingToPaths::new();

    drawing.circle(100.0, 200.0, 50.0);
    drawing.stroke_color(Color::Rgba(0.1, 0.2, 0.3, 0.4));
    drawing.fill_color(Color::Rgba(0.3, 0.4, 0.5, 0.6));
    drawing.fill();

    let paths               = drawing_to_path.draw(drawing).collect::<Vec<_>>();

    assert!(paths.len() == 1);

    let circle          = PathComponent::components_from_path(&paths[0]);
    let circle_bounds   = circle[0].bounding_box::<(Coord2, Coord2)>();
    
    assert!(circle.len() == 1);
    assert!(circle_bounds.0.distance_to(&Coord2(50.0, 150.0)) < 0.1);
    assert!(circle_bounds.1.distance_to(&Coord2(150.0, 250.0)) < 0.1);
}
