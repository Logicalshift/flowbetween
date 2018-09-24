use flo_curves::*;
use flo_curves::arc::*;
use flo_curves::bezier::*;
use flo_curves::bezier::path::*;

use std::f64;

#[test]
fn awkward_line_intersects_circle() {
    let center = Coord2(5.0, 5.0);
    let radius = 4.0;

    // Create a path from a circle
    let circle: SimpleBezierPath = Circle::new(center, radius).to_path();

    // Line from the center to the edge
    let line            = (Coord2(5.0, 5.0), Coord2(5.0, 9.5));
    let intersection    = path_intersects_line(&circle, &line).collect::<Vec<_>>();

    // This should be an intersection (straight up from the center)
    assert!(intersection.len() == 1);

    // Line from the center to the edge
    let line            = (Coord2(5.0, 5.0), Coord2(5.0, 0.5));
    let intersection    = path_intersects_line(&circle, &line).collect::<Vec<_>>();

    // This should be an intersection (straight down from the center)
    assert!(intersection.len() == 1);

    // Line from the center to the edge
    let line            = (Coord2(5.0, 5.0), Coord2(4.999999999999999, 9.5));
    let intersection    = path_intersects_line(&circle, &line).collect::<Vec<_>>();

    // This should be an intersection (almost straight up from the center)
    assert!(intersection.len() == 1);
}

#[test]
fn line_intersects_circle() {
    let center = Coord2(5.0, 5.0);
    let radius = 4.0;

    // Create a path from a circle
    let circle: SimpleBezierPath = Circle::new(center, radius).to_path();

    // Try various angles from the center to intersect it. We use a line representing a circle juuust wider than the circular path
    let length = 4.01;

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
        assert!(intersection.len() == 1);

        if intersection.len() > 0 {
            let intersection    = intersection[0];
            let intersect_point = circle_sections[intersection.0].point_at_pos(intersection.1);

            assert!(expected.distance_to(&intersect_point).abs() < 0.01);
        }
    }
}

#[test]
fn line_does_not_intersect_circle() {
    let center = Coord2(5.0, 5.0);
    let radius = 4.0;

    // Create a path from a circle
    let circle: SimpleBezierPath = Circle::new(center, radius).to_path();

    // Try various angles from the center to intersect it. Here we use a length < 4.0 so we should get no intersections
    let length = 3.9999;

    for angle in 0..=20 {
        let angle       = angle as f64;
        let radians     = (2.0*f64::consts::PI)*(angle/20.0);

        let target      = Coord2(radians.sin()*length, radians.cos()*length);
        let target      = target + center;

        // Should be one intersection with the circle here
        let line            = (center, target);
        let intersection    = path_intersects_line(&circle, &line).collect::<Vec<_>>();
        assert!(intersection.len() == 0);
    }
}

#[test]
fn circle_intersects_circle() {
    let center = Coord2(5.0, 5.0);
    let radius = 4.0;

    // Create a couple of circles (which should intersect at two points)
    let circle1: SimpleBezierPath = Circle::new(center, radius).to_path();
    let circle2: SimpleBezierPath = Circle::new(center + Coord2(1.0, 0.5), radius).to_path();

    // Get the intersections for these two circles
    let intersections = path_intersects_path(&circle1, &circle2, 0.5);

    // The circles should intersect at least once
    assert!(intersections.len() > 0);
    println!("{:?}", intersections);

    // Convert to curves
    let curves1 = circle1.to_curves::<Curve<_>>();
    let curves2 = circle2.to_curves::<Curve<_>>();

    // Check that the intersections are in roughly the same place in each circle
    for ((index1, t1), (index2, t2)) in intersections.iter() {
        let point1 = curves1[*index1].point_at_pos(*t1);
        let point2 = curves2[*index2].point_at_pos(*t2);

        println!("{:?} {:?} {:?}", point1, point2, point1.distance_to(&point2));
    }

    for ((index1, t1), (index2, t2)) in intersections.iter() {
        let point1 = curves1[*index1].point_at_pos(*t1);
        let point2 = curves2[*index2].point_at_pos(*t2);

        // Should be within the tolerance specified by the accuracy value
        assert!(point1.distance_to(&point2) < 0.5);
    }

    assert!(intersections.len() == 2);
}
