extern crate curves;

use curves::*;

#[test]
fn can_get_distance_between_points() {
    assert!(Coord2(1.0, 1.0).distance_to(&Coord2(1.0, 8.0)) == 7.0);
}

#[test]
fn can_find_unit_vector() {
    assert!(Coord2(0.0, 1.0).to_unit_vector() == Coord2(0.0, 1.0));
    assert!(Coord2(0.0, 2.0).to_unit_vector() == Coord2(0.0, 1.0));

    assert!(f64::abs(Coord2(4.0, 2.0).to_unit_vector().distance_to(&Coord2(0.0, 0.0))-1.0) < 0.01);
}

#[test]
fn unit_vector_of_0_0_is_0_0() {
    assert!(Coord2(0.0, 0.0).to_unit_vector() == Coord2(0.0, 0.0));
}

#[test]
fn can_get_dot_product() {
    assert!(Coord2(2.0,1.0).dot(&Coord2(3.0, 4.0)) == 10.0);
}