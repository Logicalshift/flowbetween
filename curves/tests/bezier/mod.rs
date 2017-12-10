use curves::bezier;
use curves::bezier::*;

#[test]
fn can_read_curve_points() {
    let curve = bezier::Curve::from_points((1.0, 1.0), (2.0, 2.0), (3.0, 3.0), (4.0, 4.0));

    assert!(curve.start_point() == (1.0, 1.0));
    assert!(curve.end_point() == (2.0, 2.0));
    assert!(curve.control_points() == ((3.0, 3.0), (4.0, 4.0)));
}
