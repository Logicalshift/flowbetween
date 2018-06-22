use flo_curves::*;
use flo_curves::bezier;
use flo_curves::line;

#[test]
fn find_intersection_on_straight_line() {
    // Cross that intersects at (5.0, 5.0)
    let line    = (Coord2(0.0, 0.0), Coord2(10.0, 10.0));
    let curve   = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(10.0, 0.0), Coord2(0.0, 10.0)));

    let intersections   = bezier::curve_intersects_line(&curve, &line);
    assert!(intersections.len() == 1);

    let intersect_point = curve.point_at_pos(intersections[0]);
    assert!(intersect_point.distance_to(&Coord2(5.0, 5.0)) < 0.01);
}

#[test]
fn find_intersection_on_curve() {
    let line    = (Coord2(0.0, 6.0), Coord2(10.0, 4.0));
    let curve   = bezier::Curve {
        start_point:    Coord2(0.0, 2.0),
        end_point:      Coord2(10.0, 8.0),
        control_points: (Coord2(0.0, 20.0), Coord2(10.0, -10.0))
    };

    // Find the intersections
    let intersections   = bezier::curve_intersects_line(&curve, &line);

    // Should be 3 intersections
    assert!(intersections.len() == 3);

    // Curve is symmetrical so the mid-point should be at 5,5
    assert!(curve.point_at_pos(intersections[1]).distance_to(&Coord2(5.0, 5.0)) < 0.01);

    // Other points are a bit less precise
    assert!(curve.point_at_pos(intersections[0]).distance_to(&Coord2(0.260, 5.948)) < 0.01);
    assert!(curve.point_at_pos(intersections[2]).distance_to(&Coord2(9.740, 4.052)) < 0.01);
}

#[test]
fn find_intersection_on_curve_short_line() {
    let line    = (Coord2(0.0, 6.0), Coord2(8.0, 4.4));
    let curve   = bezier::Curve {
        start_point:    Coord2(0.0, 2.0),
        end_point:      Coord2(10.0, 8.0),
        control_points: (Coord2(0.0, 20.0), Coord2(10.0, -10.0))
    };

    // Find the intersections
    let intersections   = bezier::curve_intersects_line(&curve, &line);

    // Should be 2 intersections
    assert!(intersections.len() == 2);

    assert!(curve.point_at_pos(intersections[1]).distance_to(&Coord2(5.0, 5.0)) < 0.01);
    assert!(curve.point_at_pos(intersections[0]).distance_to(&Coord2(0.260, 5.948)) < 0.01);
}

#[test]
fn dot_intersects_nothing() {
    // Line with 0 length
    let line    = (Coord2(4.0, 4.0), Coord2(4.0, 4.0));
    let curve   = bezier::Curve {
        start_point:    Coord2(0.0, 2.0),
        end_point:      Coord2(10.0, 8.0),
        control_points: (Coord2(0.0, 20.0), Coord2(10.0, -10.0))
    };

    // Find the intersections
    let intersections   = bezier::curve_intersects_line(&curve, &line);

    // Should be no intersections
    assert!(intersections.len() == 0);
}
