extern crate curves;

use curves::*;

#[test]
fn can_get_distance_between_points() {
    assert!(Coord2(1.0, 1.0).distance_to(&Coord2(1.0, 8.0)) == 7.0);
}