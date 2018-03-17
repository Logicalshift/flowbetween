use flo_curves::bezier;

#[test]
fn can_take_first_derivative() {
    assert!(bezier::derivative4(1.0, 2.0, 3.0, 4.0) == (3.0, 3.0, 3.0));
}
