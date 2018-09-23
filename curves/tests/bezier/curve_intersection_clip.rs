use flo_curves::*;
use flo_curves::line;
use flo_curves::bezier;

#[test]
fn find_intersection_on_straight_line_not_middle() {
    // Cross that intersects at (5.0, 5.0)
    let curve1  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(0.0, 0.0), Coord2(13.0, 13.0)));
    let curve2  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(9.0, 1.0), Coord2(0.0, 10.0)));

    let intersections   = bezier::curve_intersects_curve_clip(&curve1, &curve2, 0.1);
    println!("{:?} {:?}", intersections, intersections.iter().map(|(t1, t2)| (curve1.point_at_pos(*t1), curve2.point_at_pos(*t2))).collect::<Vec<_>>());
    assert!(intersections.len() != 0);

    let intersect_point = curve1.point_at_pos(intersections[0].0);
    assert!(intersect_point.distance_to(&Coord2(5.0, 5.0)) < 0.1);

    let intersect_point = curve2.point_at_pos(intersections[0].1);
    assert!(intersect_point.distance_to(&Coord2(5.0, 5.0)) < 0.1);

    assert!(intersections.len() == 1);
}

#[test]
fn find_intersection_on_straight_line_middle() {
    // Cross that intersects at (5.0, 5.0)
    let curve1  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(0.0, 0.0), Coord2(10.0, 10.0)));
    let curve2  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(10.0, 0.0), Coord2(0.0, 10.0)));

    let intersections   = bezier::curve_intersects_curve_clip(&curve1, &curve2, 0.1);
    println!("{:?} {:?}", intersections, intersections.iter().map(|(t1, t2)| (curve1.point_at_pos(*t1), curve2.point_at_pos(*t2))).collect::<Vec<_>>());
    assert!(intersections.len() != 0);

    let intersect_point = curve1.point_at_pos(intersections[0].0);
    assert!(intersect_point.distance_to(&Coord2(5.0, 5.0)) < 0.1);

    let intersect_point = curve2.point_at_pos(intersections[0].1);
    assert!(intersect_point.distance_to(&Coord2(5.0, 5.0)) < 0.1);

    assert!(intersections.len() == 1);
}

#[test]
fn find_intersection_on_straight_line_start() {
    // Intersection at the start of two curves
    let curve1  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(5.0, 5.0), Coord2(10.0, 10.0)));
    let curve2  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(5.0, 5.0), Coord2(0.0, 10.0)));

    let intersections   = bezier::curve_intersects_curve_clip(&curve1, &curve2, 0.1);
    assert!(intersections.len() != 0);

    let intersect_point = curve1.point_at_pos(intersections[0].0);
    assert!(intersections[0].0 < 0.01);
    assert!(intersect_point.distance_to(&Coord2(5.0, 5.0)) < 0.1);

    let intersect_point = curve2.point_at_pos(intersections[0].1);
    assert!(intersections[0].1 < 0.01);
    assert!(intersect_point.distance_to(&Coord2(5.0, 5.0)) < 0.1);

    assert!(intersections.len() == 1);
}

#[test]
fn find_intersection_on_straight_line_end() {
    // Intersection at the start of two curves
    let curve1  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(10.0, 10.0), Coord2(5.0, 5.0)));
    let curve2  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(0.0, 10.0), Coord2(5.0, 5.0)));

    let intersections   = bezier::curve_intersects_curve_clip(&curve1, &curve2, 0.1);
    assert!(intersections.len() != 0);

    let intersect_point = curve1.point_at_pos(intersections[0].0);
    assert!(intersections[0].0 > 0.99);
    assert!(intersect_point.distance_to(&Coord2(5.0, 5.0)) < 0.1);

    let intersect_point = curve2.point_at_pos(intersections[0].1);
    assert!(intersections[0].1 > 0.99);
    assert!(intersect_point.distance_to(&Coord2(5.0, 5.0)) < 0.1);

    assert!(intersections.len() == 1);
}

#[test]
fn find_intersection_on_straight_line_end_to_start() {
    // Intersection at the start of two curves
    let curve1  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(10.0, 10.0), Coord2(5.0, 5.0)));
    let curve2  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(5.0, 5.0), Coord2(0.0, 10.0)));

    let intersections   = bezier::curve_intersects_curve_clip(&curve1, &curve2, 0.1);
    assert!(intersections.len() != 0);

    let intersect_point = curve1.point_at_pos(intersections[0].0);
    assert!(intersections[0].0 > 0.99);
    assert!(intersect_point.distance_to(&Coord2(5.0, 5.0)) < 0.1);

    let intersect_point = curve2.point_at_pos(intersections[0].1);
    assert!(intersections[0].1 < 0.01);
    assert!(intersect_point.distance_to(&Coord2(5.0, 5.0)) < 0.1);

    assert!(intersections.len() == 1);
}

#[test]
fn find_intersection_on_straight_line_near_end() {
    // Intersection at the start of two curves
    let curve1  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(10.0, 10.0), Coord2(4.9, 5.1)));
    let curve2  = line::line_to_bezier::<_, bezier::Curve<_>>(&(Coord2(0.0, 10.0), Coord2(5.1, 4.9)));

    let intersections   = bezier::curve_intersects_curve_clip(&curve1, &curve2, 0.01);
    println!("{:?} {:?}", intersections, intersections.iter().map(|(t1, t2)| (curve1.point_at_pos(*t1), curve2.point_at_pos(*t2))).collect::<Vec<_>>());

    assert!(intersections.len() != 0);
    assert!(intersections.len() == 1);
}

#[test]
fn find_intersections_on_curve() {
    //
    // Two curves with three intersections
    //
    // Intersection points approx:
    //
    // Coord2(81.78, 109.88)
    // Coord2(133.16, 167.13)
    // Coord2(179.87, 199.67)
    //
    let curve1  = bezier::Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 220.0), Coord2(90.0, 30.0), Coord2(40.0, 140.0));
    let curve2  = bezier::Curve::from_points(Coord2(5.0, 150.0), Coord2(210.0, 190.0), Coord2(180.0, 20.0), Coord2(80.0, 250.0));

    let intersections   = bezier::curve_intersects_curve_clip(&curve1, &curve2, 0.1);
    println!("{:?} {:?}", intersections, intersections.iter().map(|(t1, t2)| (curve1.point_at_pos(*t1), curve2.point_at_pos(*t2))).collect::<Vec<_>>());

    // All intersections should be approximately the same location
    for intersect in intersections.iter() {
        let point1 = curve1.point_at_pos(intersect.0);
        let point2 = curve2.point_at_pos(intersect.1);

        assert!(point1.distance_to(&point2) < 1.0);
        assert!(point1.distance_to(&point2) < 0.1);
    }

    // Three intersections
    assert!(intersections.len() == 3);
}
