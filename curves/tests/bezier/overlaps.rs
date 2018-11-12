use flo_curves::bezier::*;

#[test]
fn simple_overlapping_curves() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), (Coord2(90.0, 30.0), Coord2(40.0, 140.0)), Coord2(220.0, 220.0));
    let section = curve1.section(0.3333, 0.6666);

    let overlaps = overlapping_region(&curve1, &section).unwrap();

    assert!(((overlaps.0).0 - 0.3333).abs() < 0.001);
    assert!(((overlaps.0).1 - 0.6666).abs() < 0.001);
}

#[test]
fn simple_overlapping_curves_curve2_larger() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), (Coord2(90.0, 30.0), Coord2(40.0, 140.0)), Coord2(220.0, 220.0));
    let section = curve1.section(0.3333, 0.6666);

    let overlaps = overlapping_region(&section, &curve1).unwrap();

    assert!(((overlaps.0).0 - 0.0).abs() < 0.001);
    assert!(((overlaps.0).1 - 1.0).abs() < 0.001);

    assert!(((overlaps.1).0 - 0.3333).abs() < 0.001);
    assert!(((overlaps.1).1 - 0.6666).abs() < 0.001);
}

#[test]
fn simple_overlapping_curves_same() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), (Coord2(90.0, 30.0), Coord2(40.0, 140.0)), Coord2(220.0, 220.0));
    let section = Curve::from_points(Coord2(10.0, 100.0), (Coord2(90.0, 30.0), Coord2(40.0, 140.0)), Coord2(220.0, 220.0));

    let overlaps = overlapping_region(&curve1, &section).unwrap();

    assert!(((overlaps.0).0 - 0.0).abs() < 0.001);
    assert!(((overlaps.0).1 - 1.0).abs() < 0.001);
}

#[test]
fn simple_overlapping_curves_reversed() {
    let curve1  = Curve::from_points(Coord2(220.0, 220.0), (Coord2(90.0, 30.0), Coord2(40.0, 140.0)), Coord2(10.0, 100.0));
    let section = Curve::from_points(Coord2(10.0, 100.0), (Coord2(40.0, 140.0), Coord2(90.0, 30.0)), Coord2(220.0, 220.0));

    let overlaps = overlapping_region(&curve1, &section).unwrap();

    assert!(((overlaps.0).0 - 1.0).abs() < 0.001);
    assert!(((overlaps.0).1 - 0.0).abs() < 0.001);
}

#[test]
fn simple_overlapping_lines() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), (Coord2(30.0, 100.0), Coord2(200.0, 100.0)), Coord2(220.0, 100.0));
    let section = curve1.section(0.3333, 0.6666);

    let overlaps = overlapping_region(&curve1, &section).unwrap();

    assert!(((overlaps.0).0 - 0.3333).abs() < 0.001);
    assert!(((overlaps.0).1 - 0.6666).abs() < 0.001);
}

#[test]
fn overlapping_lines_same() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), (Coord2(30.0, 100.0), Coord2(200.0, 100.0)), Coord2(220.0, 100.0));
    let section = Curve::from_points(Coord2(10.0, 100.0), (Coord2(30.0, 100.0), Coord2(200.0, 100.0)), Coord2(220.0, 100.0));

    let overlaps = overlapping_region(&curve1, &section).unwrap();

    assert!(((overlaps.0).0 - 0.0).abs() < 0.001);
    assert!(((overlaps.0).1 - 1.0).abs() < 0.001);
}

#[test]
fn overlapping_lines_with_different_t_values() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), (Coord2(30.0, 100.0), Coord2(200.0, 100.0)), Coord2(220.0, 100.0));
    let section = Curve::from_points(Coord2(10.0, 100.0), (Coord2(50.0, 100.0), Coord2(180.0, 100.0)), Coord2(220.0, 100.0));

    let overlaps = overlapping_region(&curve1, &section).unwrap();

    assert!(((overlaps.0).0 - 0.0).abs() < 0.001);
    assert!(((overlaps.0).1 - 1.0).abs() < 0.001);
}

#[test]
fn overlaps_with_known_curve_1() {
    // These curves should overlap
    let curve1 = Curve::from_points(Coord2(346.69864, 710.2048), (Coord2(350.41446, 706.8076), Coord2(353.61026, 702.4266)), Coord2(356.28525, 698.20306));
    let curve2 = Curve::from_points(Coord2(350.22574, 706.551), (Coord2(354.72943, 701.2933), Coord2(358.0882, 695.26)), Coord2(361.0284, 690.2511));

    // They currently don't
    assert!(curve1.t_for_point(&curve2.start_point()).is_some() || curve2.t_for_point(&curve1.start_point()).is_some());
    assert!(curve1.t_for_point(&curve2.end_point()).is_some() || curve2.t_for_point(&curve1.end_point()).is_some());

    assert!(!overlapping_region(&curve1, &curve2).is_some());
}

#[test]
fn overlaps_with_known_curve_2() {
    // These curves should overlap
    let curve1 = Curve::from_points(Coord2(305.86907958984375, 882.2529296875), (Coord2(305.41015625, 880.7345581054688), Coord2(303.0707092285156, 879.744140625)), Coord2(298.0640869140625, 875.537353515625));
    let curve2 = Curve::from_points(Coord2(302.7962341308594, 879.1681518554688), (Coord2(299.5769348144531, 876.8582763671875), Coord2(297.1976318359375, 874.7939453125)), Coord2(301.4282531738281, 878.26220703125));

    // They currently don't
    assert!(!curve1.t_for_point(&curve2.start_point()).is_some() || curve2.t_for_point(&curve1.start_point()).is_some());
    assert!(curve1.t_for_point(&curve2.end_point()).is_some() || curve2.t_for_point(&curve1.end_point()).is_some());

    assert!(!overlapping_region(&curve1, &curve2).is_some());
}

#[test]
fn overlaps_with_known_curve_3() {
    // These curves should overlap
    let curve1 = Curve::from_points(Coord2(510.6888427734375, 684.9293212890625), (Coord2(511.68206787109375, 683.7874145507813), Coord2(512.7827758789063, 682.6954345703125)), Coord2(513.9757080078125, 681.668212890625));
    let curve2 = Curve::from_points(Coord2(510.6888427734375, 684.9293212890625), (Coord2(511.66473388671875, 683.8077392578125), Coord2(512.7447509765625, 682.73388671875)), Coord2(513.9143676757813, 681.7202758789063));

    assert!(overlapping_region(&curve1, &curve2).is_some());
    assert!(overlapping_region(&curve2, &curve1).is_some());
}
