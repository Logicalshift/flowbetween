use flo_curves::bezier::*;

#[test]
fn simple_overlapping_curves() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 220.0), Coord2(90.0, 30.0), Coord2(40.0, 140.0));
    let section = curve1.section(0.3333, 0.6666);

    let overlaps = overlapping_region(&curve1, &section).unwrap();

    assert!(((overlaps.0).0 - 0.3333).abs() < 0.001);
    assert!(((overlaps.0).1 - 0.6666).abs() < 0.001);
}

#[test]
fn simple_overlapping_curves_curve2_larger() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 220.0), Coord2(90.0, 30.0), Coord2(40.0, 140.0));
    let section = curve1.section(0.3333, 0.6666);

    let overlaps = overlapping_region(&section, &curve1).unwrap();

    assert!(((overlaps.0).0 - 0.0).abs() < 0.001);
    assert!(((overlaps.0).1 - 1.0).abs() < 0.001);

    assert!(((overlaps.1).0 - 0.3333).abs() < 0.001);
    assert!(((overlaps.1).1 - 0.6666).abs() < 0.001);
}

#[test]
fn simple_overlapping_curves_same() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 220.0), Coord2(90.0, 30.0), Coord2(40.0, 140.0));
    let section = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 220.0), Coord2(90.0, 30.0), Coord2(40.0, 140.0));

    let overlaps = overlapping_region(&curve1, &section).unwrap();

    assert!(((overlaps.0).0 - 0.0).abs() < 0.001);
    assert!(((overlaps.0).1 - 1.0).abs() < 0.001);
}

#[test]
fn simple_overlapping_curves_reversed() {
    let curve1  = Curve::from_points(Coord2(220.0, 220.0), Coord2(10.0, 100.0), Coord2(90.0, 30.0), Coord2(40.0, 140.0));
    let section = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 220.0), Coord2(40.0, 140.0), Coord2(90.0, 30.0));

    let overlaps = overlapping_region(&curve1, &section).unwrap();

    assert!(((overlaps.0).0 - 1.0).abs() < 0.001);
    assert!(((overlaps.0).1 - 0.0).abs() < 0.001);
}

#[test]
fn simple_overlapping_lines() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 100.0), Coord2(30.0, 100.0), Coord2(200.0, 100.0));
    let section = curve1.section(0.3333, 0.6666);

    let overlaps = overlapping_region(&curve1, &section).unwrap();

    assert!(((overlaps.0).0 - 0.3333).abs() < 0.001);
    assert!(((overlaps.0).1 - 0.6666).abs() < 0.001);
}

#[test]
fn overlapping_lines_same() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 100.0), Coord2(30.0, 100.0), Coord2(200.0, 100.0));
    let section = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 100.0), Coord2(30.0, 100.0), Coord2(200.0, 100.0));

    let overlaps = overlapping_region(&curve1, &section).unwrap();

    assert!(((overlaps.0).0 - 0.0).abs() < 0.001);
    assert!(((overlaps.0).1 - 1.0).abs() < 0.001);
}

#[test]
fn overlapping_lines_with_different_t_values() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 100.0), Coord2(30.0, 100.0), Coord2(200.0, 100.0));
    let section = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 100.0), Coord2(50.0, 100.0), Coord2(180.0, 100.0));

    let overlaps = overlapping_region(&curve1, &section).unwrap();

    assert!(((overlaps.0).0 - 0.0).abs() < 0.001);
    assert!(((overlaps.0).1 - 1.0).abs() < 0.001);
}
