use super::*;
use curves::bezier;

#[test]
fn basis_at_t0_is_w1() {
    assert!(bezier::basis(0.0, 2.0, 3.0, 4.0, 5.0) == 2.0);
}

#[test]
fn basis_at_t1_is_w4() {
    assert!(bezier::basis(1.0, 2.0, 3.0, 4.0, 5.0) == 5.0);
}

#[test]
fn basis_agrees_with_de_casteljau() {
    for x in 0..100 {
        let t               = (x as f32)/100.0;

        let basis           = bezier::basis(t, 2.0, 3.0, 4.0, 5.0);
        let de_casteljau    = bezier::de_casteljau4(t, 2.0, 3.0, 4.0, 5.0);

        assert!(approx_equal(basis, de_casteljau));
    }
}
