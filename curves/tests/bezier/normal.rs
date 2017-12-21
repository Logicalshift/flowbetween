use curves::*;
use curves::bezier;
use curves::bezier::NormalCurve;

#[test]
fn normal_for_line_is_straight_up() {
    let line    = bezier::Curve::from_points(Coord2(0.0,0.0), Coord2(10.0, 0.0), Coord2(3.0, 0.0), Coord2(7.0, 0.0));
    let normal  = line.normal_at_pos(0.5);

    // Normal should be a line facing up
    assert!(normal.x().abs() < 0.01);
    assert!(normal.y() > 0.01);
}

#[test]
fn normal_at_start_of_curve_matches_control_points() {
    let line    = bezier::Curve::from_points(Coord2(0.0,0.0), Coord2(10.0, 0.0), Coord2(0.0, 1.0), Coord2(7.0, 0.0));
    let normal  = line.normal_at_pos(0.0);

    // Normal should be a facing left
    assert!(normal.x() < 0.0);
    assert!(normal.y().abs() < 0.01);
}

