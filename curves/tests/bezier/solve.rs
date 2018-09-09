use flo_curves::bezier::*;

#[test]
fn basis_solve_middle() {
    assert!((solve_basis_for_t(0.0, 0.33, 0.66, 1.0, 0.5)[0]-0.5).abs() < 0.01);
    assert!((solve_basis_for_t(0.0, 1.0, 2.0, 3.0, 1.5)[0]-0.5).abs() < 0.01);
}

#[test]
fn basis_solve_many() {
    fn test_for(w1: f64, w2: f64, w3: f64, w4: f64) {
        for p in 0..=16 {
            // Pick a point between w1 and w4
            let p = ((p as f64)/16.0)*(w4-w1) + w1;

            // Solve for t values
            let t_values = solve_basis_for_t(w1, w2, w3, w4, p);

            // Computing the points for these values should result in a valid curve
            let pos_for_t = t_values.iter()
                .map(|t| basis(*t, w1, w2, w3, w4))
                .collect::<Vec<_>>();

            // Should all evaluate to positions on the curve
            pos_for_t.iter().for_each(|pos| assert!((pos-p).abs() < 0.01));
        }
    }

    test_for(0.0, 0.33, 0.66, 1.0);
    test_for(2.0, 3.0, 4.0, 5.0);
    test_for(2.0, -1.0, 5.0, 3.0);
}
