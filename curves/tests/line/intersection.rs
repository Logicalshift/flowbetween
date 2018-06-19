use flo_curves::*;
use flo_curves::line::*;

#[test]
fn intersection_at_0_0() {
    assert!(line_intersects_line(&(Coord2(-1.0, 0.0), Coord2(1.0, 0.0)), &(Coord2(0.0, 1.0), Coord2(0.0, -1.0))).unwrap().distance_to(&Coord2(0.0, 0.0)) < 0.01);
}

#[test]
fn intersection_at_other_point() {
    assert!(line_intersects_line(&(Coord2(10.0, 20.0), Coord2(50.0, 60.0)), &(Coord2(10.0, 45.0), Coord2(50.0, 35.0))).unwrap().distance_to(&Coord2(30.0, 40.0)) < 0.01);
}

#[test]
fn no_intersection() {
    assert!(line_intersects_line(&(Coord2(12.0, 13.0), Coord2(24.0, 30.0)), &(Coord2(1.0, 1.0), Coord2(0.0, -1.0))) == None);
}
