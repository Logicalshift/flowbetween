use flo_curves::*;
use flo_curves::arc::*;
use flo_curves::bezier::*;
use flo_curves::bezier::path::*;

use std::f64;

#[test]
fn line_intersects_circle() {
    let center = Coord2(5.0, 5.0);
    let radius = 4.0;

    // Create a path from a circle
    let circle: SimpleBezierPath = Circle::new(center, radius).to_path();

    // Try various angles from the center to intersect it
    let length = 4.5;

    let circle_sections = circle.to_curves::<Curve<_>>();

    for angle in 0..=20 {
        let angle       = angle as f64;
        let radians     = (2.0*f64::consts::PI)*(angle/20.0);

        let target      = Coord2(radians.sin()*length, radians.cos()*length);
        let target      = target + center;

        let expected    = Coord2(radians.sin()*radius, radians.cos()*radius);
        let expected    = expected + center;

        // Should be one intersection with the circle here
        let line            = (center, target);
        let intersection    = path_intersects_line(&circle, &line).collect::<Vec<_>>();
        println!("{:?} {:?}", intersection, target);
        assert!(intersection.len() == 1);

        let intersection    = intersection[0];
        let intersect_point = circle_sections[intersection.0].point_at_pos(intersection.1);

        println!("{:?} {:?}", expected, intersect_point);
        assert!(expected.distance_to(&intersect_point).abs() < 0.01);
    }

    assert!(false);
}
