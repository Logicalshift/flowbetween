use curves::bezier;

#[test]
fn basis_at_t0_is_w1() {
    assert!(bezier::basis(0.0, 2.0, 3.0, 4.0, 5.0) == 2.0);
}

#[test]
fn basis_at_t1_is_w4() {
    assert!(bezier::basis(1.0, 2.0, 3.0, 4.0, 5.0) == 5.0);
}
