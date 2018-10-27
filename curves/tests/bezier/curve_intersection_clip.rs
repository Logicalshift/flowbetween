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

#[test]
fn intersections_with_nearby_curves_1() {
    let curve1 = bezier::Curve::from_points(Coord2(346.69864, 710.2048), Coord2(356.28525, 698.20306), Coord2(350.41446, 706.8076), Coord2(353.61026, 702.4266));
    let curve2 = bezier::Curve::from_points(Coord2(350.22574, 706.551), Coord2(361.0284, 690.2511), Coord2(354.72943, 701.2933), Coord2(358.0882, 695.26));

    let intersections   = bezier::curve_intersects_curve_clip(&curve1, &curve2, 0.01);

    println!("{:?}", intersections);

    assert!(intersections.len() <= 9);
}

#[test]
fn intersections_with_nearby_curves_2() {
    let curve1 = bezier::Curve::from_points(Coord2(305.86907958984375, 882.2529296875), Coord2(298.0640869140625, 875.537353515625), Coord2(305.41015625, 880.7345581054688), Coord2(303.0707092285156, 879.744140625));
    let curve2 = bezier::Curve::from_points(Coord2(302.7962341308594, 879.1681518554688), Coord2(301.4282531738281, 878.26220703125), Coord2(299.5769348144531, 876.8582763671875), Coord2(297.1976318359375, 874.7939453125));

    let intersections   = bezier::curve_intersects_curve_clip(&curve1, &curve2, 0.01);
    println!("{:?}", intersections);

    assert!(intersections.len() <= 9);
}

#[test]
fn intersections_with_nearby_curves_3() {
    let curve1 = bezier::Curve::from_points(Coord2(304.6919250488281, 880.6288452148438), Coord2(296.8869323730469, 873.9132690429688), Coord2(304.2330017089844, 879.1104736328125), Coord2(301.8935546875, 878.1200561523438));
    let curve2 = bezier::Curve::from_points(Coord2(301.61907958984375, 877.5440673828125), Coord2(296.0204772949219, 873.1698608398438), Coord2(300.2510986328125, 876.6381225585938), Coord2(298.3997802734375, 875.2341918945313));

    let intersections   = bezier::curve_intersects_curve_clip(&curve1, &curve2, 0.01);
    println!("{:?}", intersections);

    // assert!(intersections.len() <= 9);
}

#[test]
fn intersections_with_nearby_curves_4() {
    let curve1 = bezier::Curve::from_points(Coord2(436.15716552734375, 869.3236083984375), Coord2(490.6786804199219, 849.5614624023438), Coord2(444.5263671875, 869.2921752929688), Coord2(480.9628601074219, 854.3709106445313));
    let curve2 = bezier::Curve::from_points(Coord2(462.5539855957031, 861.322021484375), Coord2(462.3448486328125, 861.8137817382813), Coord2(462.4580078125, 861.4293823242188), Coord2(462.3710021972656, 861.5908813476563));

    let intersections   = bezier::curve_intersects_curve_clip(&curve1, &curve2, 0.01);
    println!("{:?}", intersections);

    assert!(intersections.len() <= 9);
}

#[test]
fn intersection_curve_1() {
    let curve1 = bezier::Curve::from_points(Coord2(252.08901977539063, 676.4180908203125), Coord2(244.31190490722656, 686.1041259765625), Coord2(244.0195770263672, 679.6658935546875), Coord2(244.11508178710938, 682.8816528320313));
    let curve2 = bezier::Curve::from_points(Coord2(244.31190490722656, 686.1041259765625), Coord2(265.2398376464844, 618.4223022460938), Coord2(250.65411376953125, 661.4817504882813), Coord2(255.51109313964844, 635.5418701171875));

    let intersections = bezier::curve_intersects_curve_clip(&curve1, &curve2, 0.01);
    println!("{:?}", intersections);
    assert!(intersections.len() != 0);
    assert!(intersections.len() != 1);
    assert!(intersections.len() == 2);

    assert!(curve1.point_at_pos(intersections[0].0).distance_to(&curve2.point_at_pos(intersections[0].1)) < 0.01);
    assert!(curve1.point_at_pos(intersections[1].0).distance_to(&curve2.point_at_pos(intersections[1].1)) < 0.01);

    let intersections = bezier::curve_intersects_curve_clip(&curve2, &curve1, 0.01);
    println!("{:?}", intersections);
    assert!(intersections.len() != 0);
    assert!(intersections.len() == 1);

    // TODO: should be two intersections (one at the start and one later on, but we only get one at the moment)

    assert!(curve2.point_at_pos(intersections[0].0).distance_to(&curve1.point_at_pos(intersections[0].1)) < 0.01);
    assert!(intersections[0].0 > 0.01 && intersections[0].0 < 0.99);
    assert!(intersections[0].1 > 0.01 && intersections[0].1 < 0.99);
}

#[test]
fn intersection_curve_2() {
    let curve1 = bezier::Curve::from_points(Coord2(248.42221069335938, 678.5138549804688), Coord2(258.2634582519531, 745.7745361328125), Coord2(240.33773803710938, 703.49462890625), Coord2(246.20928955078125, 728.5226440429688));
    let curve2 = bezier::Curve::from_points(Coord2(240.6450958251953, 688.1998901367188), Coord2(248.42221069335938, 678.5138549804688), Coord2(248.51101684570313, 684.6644897460938), Coord2(248.41787719726563, 681.5728759765625));

    let intersections = bezier::curve_intersects_curve_clip(&curve1, &curve2, 0.01);
    println!("{:?}", intersections);
    assert!(intersections.len() != 0);
    assert!(intersections.len() != 1);
    assert!(intersections.len() == 2);

    assert!(curve1.point_at_pos(intersections[0].0).distance_to(&curve2.point_at_pos(intersections[0].1)) < 0.01);
    assert!(curve1.point_at_pos(intersections[1].0).distance_to(&curve2.point_at_pos(intersections[1].1)) < 0.01);

    let intersections = bezier::curve_intersects_curve_clip(&curve2, &curve1, 0.01);
    println!("{:?}", intersections);
    assert!(intersections.len() != 0);
    assert!(intersections.len() == 1);

    // TODO: should be two intersections (one at the start and one later on, but we only get one at the moment)

    assert!(curve2.point_at_pos(intersections[0].0).distance_to(&curve1.point_at_pos(intersections[0].1)) < 0.01);
}
