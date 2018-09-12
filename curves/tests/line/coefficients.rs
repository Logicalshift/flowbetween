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
