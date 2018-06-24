use flo_curves::*;
use flo_curves::arc::*;
use flo_curves::bezier::path::*;

#[test]
fn circle_path_bounds() {
    let center = Coord2(5.0, 5.0);
    let radius = 4.0;

    // Create a path from a circle
    let circle: SimpleBezierPath = Circle::new(center, radius).to_path();

    let bounds = circle.bounds();

    assert!(bounds.0.distance_to(&Coord2(1.0, 1.0)) < 0.1);
    assert!(bounds.1.distance_to(&Coord2(9.0, 9.0)) < 0.1);
}
