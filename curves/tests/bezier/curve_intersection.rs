use flo_curves::*;
use flo_curves::line;
use flo_curves::bezier;

#[test]
fn find_intersection_on_straight_line_not_middle() {
    // Cross that intersects at (5.0, 5.0)
    let curve1  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(0.0, 0.0), Coord2(10.0, 10.0)));
    let curve2  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(13.0, 0.0), Coord2(0.0, 13.0)));

    let intersections   = bezier::curve_intersects_curve(&curve1, &curve2, 0.1);
    assert!(intersections.len() != 0);

    let intersect_point = curve1.point_at_pos(intersections[0].0);
    assert!(intersect_point.distance_to(&Coord2(5.0, 5.0)) < 0.1);

    let intersect_point = curve2.point_at_pos(intersections[0].1);
    assert!(intersect_point.distance_to(&Coord2(5.0, 5.0)) < 0.1);

    assert!(intersections.len() == 1);
}
