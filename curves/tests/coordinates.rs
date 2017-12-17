extern crate curves;

use curves::*;

#[test]
fn can_get_distance_between_points() {
    assert!(Coord2(1.0, 1.0).distance_to(&Coord2(1.0, 8.0)) == 7.0);
}


#[test]
fn can_normalize() {
    assert!(Coord2(0.0, 1.0).normalize() == Coord2(0.0, 1.0));
    assert!(Coord2(0.0, 2.0).normalize() == Coord2(0.0, 1.0));

    assert!(f32::abs(Coord2(4.0, 2.0).normalize().distance_to(&Coord2(0.0, 0.0))-1.0) < 0.01);
}
