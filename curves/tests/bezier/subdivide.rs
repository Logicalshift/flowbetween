use super::*;

use curves::bezier;

#[test]
fn can_subdivide_1() {
    // Initial curve
    let (w1, w2, w3, w4) = (1.0, 2.0, 3.0, 4.0);

    // Subdivide at 33%, creating two curves
    let ((wa1, wa2, wa3, wa4), (_wb1, _wb2, _wb3, _wb4)) = bezier::subdivide4(0.33, w1, w2, w3, w4);

    // Check that the original curve corresponds to the basis function for wa
    for x in 0..100 {
        let t = (x as f32)/100.0;

        let original    = bezier::basis(t*0.33, w1, w2, w3, w4);
        let subdivision = bezier::basis(t, wa1, wa2, wa3, wa4);

        assert!(approx_equal(original, subdivision));
    }
}

#[test]
fn can_subdivide_2() {
    // Initial curve
    let (w1, w2, w3, w4) = (1.0, 2.0, 3.0, 4.0);

    // Subdivide at 33%, creating two curves
    let ((_wa1, _wa2, _wa3, _wa4), (wb1, wb2, wb3, wb4)) = bezier::subdivide4(0.33, w1, w2, w3, w4);

    // Check that the original curve corresponds to the basis function for wb
    for x in 0..100 {
        let t = (x as f32)/100.0;

        let original    = bezier::basis(0.33+(t*(1.0-0.33)), w1, w2, w3, w4);
        let subdivision = bezier::basis(t, wb1, wb2, wb3, wb4);

        assert!(approx_equal(original, subdivision));
    }
}
