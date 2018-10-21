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

#[test]
fn solve_t_for_pos() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 220.0), Coord2(90.0, 30.0), Coord2(40.0, 140.0));

    let point_at_one_third  = curve1.point_at_pos(0.3333);
    let solved              = curve1.t_for_point(&point_at_one_third);

    assert!(solved.is_some());
    assert!((solved.unwrap()-0.3333).abs() < 0.001);
}

#[test]
fn solve_t_for_start() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 220.0), Coord2(90.0, 30.0), Coord2(40.0, 140.0));

    let solved  = curve1.t_for_point(&Coord2(10.0, 100.0));

    assert!(solved.is_some());
    assert!((solved.unwrap()-0.0).abs() < 0.001);
}

#[test]
fn solve_t_for_end() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 220.0), Coord2(90.0, 30.0), Coord2(40.0, 140.0));

    let solved  = curve1.t_for_point(&Coord2(220.0, 220.0));

    assert!(solved.is_some());
    assert!((solved.unwrap()-1.0).abs() < 0.001);
}

#[test]
fn solve_t_for_many_positions() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 220.0), Coord2(90.0, 30.0), Coord2(40.0, 140.0));

    for p in 0..10 {
        let p       = (p as f64)/10.0;
        let point   = curve1.point_at_pos(p);
        let solved  = curve1.t_for_point(&point);

        assert!(solved.is_some());
        assert!((solved.unwrap()-p).abs() < 0.001);
    }
}

#[test]
fn solve_t_for_out_of_bounds() {
    let curve1  = Curve::from_points(Coord2(10.0, 100.0), Coord2(220.0, 220.0), Coord2(90.0, 30.0), Coord2(40.0, 140.0));

    let solved  = curve1.t_for_point(&Coord2(45.0, 23.0));
    assert!(solved.is_none());
}
