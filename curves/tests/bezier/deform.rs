use flo_curves::*;
use flo_curves::bezier;

#[test]
fn deform_line_upwards() {
    let curve       = bezier::Curve::from_points(Coord2(0.0, 0.0), (Coord2(2.5, 0.0), Coord2(7.5, 0.0)), Coord2(10.0, 0.0));
    let deformed    = bezier::move_point::<_, _, bezier::Curve<_>>(&curve, 0.5, &Coord2(0.0, 4.0));

    let original_point  = curve.point_at_pos(0.5);
    let new_point       = deformed.point_at_pos(0.5);

    let offset          = new_point - original_point;

    assert!((offset.x()-0.0).abs() < 0.01);
    assert!((offset.y()-4.0).abs() < 0.01);

    assert!(curve.point_at_pos(0.0).distance_to(&deformed.point_at_pos(0.0)) < 0.01);
    assert!(curve.point_at_pos(1.0).distance_to(&deformed.point_at_pos(1.0)) < 0.01);
}

#[test]
fn deform_curve_at_halfway_point() {
    let curve       = bezier::Curve::from_points(Coord2(10.0, 20.0), (Coord2(0.0, 15.0), Coord2(16.0, 30.0)), Coord2(20.0, 15.0));
    let deformed    = bezier::move_point::<_, _, bezier::Curve<_>>(&curve, 0.5, &Coord2(3.0, 4.0));

    let original_point  = curve.point_at_pos(0.5);
    let new_point       = deformed.point_at_pos(0.5);

    let offset          = new_point - original_point;

    assert!((offset.x()-3.0).abs() < 0.01);
    assert!((offset.y()-4.0).abs() < 0.01);

    assert!(curve.point_at_pos(0.0).distance_to(&deformed.point_at_pos(0.0)) < 0.01);
    assert!(curve.point_at_pos(1.0).distance_to(&deformed.point_at_pos(1.0)) < 0.01);
}

#[test]
fn deform_curve_at_other_point() {
    let t           = 0.32;
    let curve       = bezier::Curve::from_points(Coord2(10.0, 20.0), (Coord2(0.0, 15.0), Coord2(16.0, 30.0)), Coord2(20.0, 15.0));
    let deformed    = bezier::move_point::<_, _, bezier::Curve<_>>(&curve, t, &Coord2(3.0, 4.0));

    let original_point  = curve.point_at_pos(t);
    let new_point       = deformed.point_at_pos(t);

    let offset          = new_point - original_point;

    assert!((offset.x()-3.0).abs() < 0.01);
    assert!((offset.y()-4.0).abs() < 0.01);

    assert!(curve.point_at_pos(0.0).distance_to(&deformed.point_at_pos(0.0)) < 0.01);
    assert!(curve.point_at_pos(1.0).distance_to(&deformed.point_at_pos(1.0)) < 0.01);
}

#[test]
fn deform_curve_at_many_other_points() {
    for t in 0..100 {
        // Won't work at 0 or 1 as these are the start and end points and don't move
        let t           = (t as f64)/100.0;
        let t           = (0.9*t)+0.05;

        let curve       = bezier::Curve::from_points(Coord2(5.0, 23.0), (Coord2(-10.0, 15.0), Coord2(26.0, 30.0)), Coord2(22.0, 17.0));
        let deformed    = bezier::move_point::<_, _, bezier::Curve<_>>(&curve, t, &Coord2(6.0, -4.0));

        let original_point  = curve.point_at_pos(t);
        let new_point       = deformed.point_at_pos(t);

        let offset          = new_point - original_point;

        assert!((offset.x()-6.0).abs() < 0.01);
        assert!((offset.y()- -4.0).abs() < 0.01);

        assert!(curve.point_at_pos(0.0).distance_to(&deformed.point_at_pos(0.0)) < 0.01);
        assert!(curve.point_at_pos(1.0).distance_to(&deformed.point_at_pos(1.0)) < 0.01);
    }
}
