use flo_curves::*;
use flo_curves::bezier;

#[test]
fn can_calculate_tangent_for_straight_line() {
    let straight_line   = bezier::Curve::from_points(Coord2(0.0, 1.0), Coord2(2.0, 3.0), Coord2(0.5, 1.5), Coord2(1.5, 2.5));
    let tangent         = bezier::Tangent::from(&straight_line);

    assert!(tangent.tangent(0.5) == Coord2(2.25, 2.25));

    assert!(tangent.tangent(0.0).x() == tangent.tangent(0.0).y());
    assert!(tangent.tangent(0.5).x() == tangent.tangent(0.5).y());
    assert!(tangent.tangent(0.7).x() == tangent.tangent(0.7).y());
    assert!(tangent.tangent(1.0).x() == tangent.tangent(1.0).y());
}
