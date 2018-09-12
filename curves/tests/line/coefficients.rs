use flo_curves::line::*;

#[test]
fn points_on_line_are_on_line_1() {
    let line        = (Coord2(2.0, 3.0), Coord2(7.0, 6.0));
    let (a, b, c)   = line_coefficients_2d(&line);

    for t in 0..=16 {
        let t       = (t as f64) / 16.0;
        let point   = line.point_at_pos(t);

        assert!((a*point.x() + b*point.y() + c).abs() < 0.001);
    }
}

#[test]
fn points_on_line_are_on_line_2() {
    let line        = (Coord2(7.0, 6.0), Coord2(2.0, 3.0));
    let (a, b, c)   = line_coefficients_2d(&line);

    for t in 0..=16 {
        let t       = (t as f64) / 16.0;
        let point   = line.point_at_pos(t);

        assert!((a*point.x() + b*point.y() + c).abs() < 0.001);
    }
}

#[test]
fn points_on_line_are_on_line_3() {
    let line        = (Coord2(2.0, 3.0), Coord2(7.0, 3.0));
    let (a, b, c)   = line_coefficients_2d(&line);

    for t in 0..=16 {
        let t       = (t as f64) / 16.0;
        let point   = line.point_at_pos(t);

        assert!((a*point.x() + b*point.y() + c).abs() < 0.001);
    }
}

#[test]
fn points_on_line_are_on_line_4() {
    let line        = (Coord2(2.0, 3.0), Coord2(2.0, 6.0));
    let (a, b, c)   = line_coefficients_2d(&line);

    for t in 0..=16 {
        let t       = (t as f64) / 16.0;
        let point   = line.point_at_pos(t);

        assert!((a*point.x() + b*point.y() + c).abs() < 0.001);
    }
}

#[test]
fn points_on_line_are_on_line_5() {
    let line        = (Coord2(2.0, 3.0), Coord2(2.0, 6.0));
    let (a, b, c)   = line.coefficients();

    for t in 0..=16 {
        let t       = (t as f64) / 16.0;
        let point   = line.point_at_pos(t);

        assert!((a*point.x() + b*point.y() + c).abs() < 0.001);
    }
}

#[test]
fn distance_from_horizontal_line() {
    let line = (Coord2(2.0, 3.0), Coord2(8.0, 3.0));

    assert!((line.distance_to(&Coord2(4.0, 3.0))).abs() < 0.001);
    assert!((line.distance_to(&Coord2(5.0, 4.0)) - 1.0).abs() < 0.001);
    assert!((line.distance_to(&Coord2(3.0, 0.0)) - -3.0).abs() < 0.001);
}

#[test]
fn distance_from_vertical_line() {
    let line = (Coord2(2.0, 3.0), Coord2(2.0, 9.0));

    assert!((line.distance_to(&Coord2(2.0, 5.0))).abs() < 0.001);
    assert!((line.distance_to(&Coord2(3.0, 4.0)) - 1.0).abs() < 0.001);
    assert!((line.distance_to(&Coord2(0.0, 0.0)) - -2.0).abs() < 0.001);
}

#[test]
fn distance_from_diagonal_line() {
    let line = (Coord2(2.0, 3.0), Coord2(5.0, 9.0));

    assert!((line.distance_to(&Coord2(3.5, 6.0))).abs() < 0.001);
    assert!((line.distance_to(&Coord2(3.0, 4.0)) - 0.4472).abs() < 0.001);
}
