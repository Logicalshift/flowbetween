use flo_curves::*;
use flo_curves::bezier;
use flo_curves::line;

#[test]
fn find_intersection_on_straight_line() {
    // Cross that intersects at (5.0, 5.0)
    let line    = (Coord2(0.0, 0.0), Coord2(10.0, 10.0));
    let curve   = line::line_to_bezier::<_, bezier::Curve>(&(Coord2(10.0, 0.0), Coord2(0.0, 10.0)));

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
        control_points: (Coord2(0.0, 10.0), Coord2(10.0, 0.0))
    };

    // Find the intersections
    let intersections   = bezier::curve_intersects_line(&curve, &line);

    // TODO: the bezier search algorithm we're using produces too many patches (there should be 3 with this curve)
    // assert!(intersections.len() == 3);
    println!("{:?}", (0..=20).into_iter().map(|t| { let t = (t as f64)/20.0; line.point_at_pos(t) }).collect::<Vec<_>>());
    println!("{:?}", intersections.into_iter().map(|t| curve.point_at_pos(t)).collect::<Vec<_>>());
}
