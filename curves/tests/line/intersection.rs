use flo_curves::*;
use flo_curves::line::*;

#[test]
fn intersection_at_0_0() {
    assert!(line_intersects_line(&(Coord2(-1.0, 0.0), Coord2(1.0, 0.0)), &(Coord2(0.0, 1.0), Coord2(0.0, -1.0))).unwrap().distance_to(&Coord2(0.0, 0.0)) < 0.01);
}

#[test]
fn intersection_at_other_point() {
    assert!(line_intersects_line(&(Coord2(10.0, 20.0), Coord2(50.0, 60.0)), &(Coord2(10.0, 45.0), Coord2(50.0, 35.0))).unwrap().distance_to(&Coord2(30.0, 40.0)) < 0.01);
}

#[test]
fn no_intersection() {
    assert!(line_intersects_line(&(Coord2(12.0, 13.0), Coord2(24.0, 30.0)), &(Coord2(1.0, 1.0), Coord2(0.0, -1.0))) == None);
}

#[test]
fn line_in_bounds() {
    let line    = (Coord2(5.0, 3.0), Coord2(7.0, 9.0));
    let bounds  = (Coord2(1.0, 1.0), Coord2(10.0, 10.0));
    let clipped = line_clip_to_bounds(&line, &bounds);

    assert!(clipped.is_some());

    let clipped = clipped.unwrap();
    assert!(clipped.0.distance_to(&Coord2(5.0, 3.0)) < 0.01);
    assert!(clipped.1.distance_to(&Coord2(7.0, 9.0)) < 0.01);
}

#[test]
fn horizontal_clipped_line() {
    let line    = (Coord2(-10.0, 4.0), Coord2(20.0, 4.0));
    let bounds  = (Coord2(1.0, 1.0), Coord2(10.0, 10.0));
    let clipped = line_clip_to_bounds(&line, &bounds);

    println!("{:?}", clipped);
    assert!(clipped.is_some());

    let clipped = clipped.unwrap();
    assert!(clipped.0.distance_to(&Coord2(1.0, 4.0)) < 0.01);
    assert!(clipped.1.distance_to(&Coord2(10.0, 4.0)) < 0.01);
}

#[test]
fn horizontal_clipped_line_inside_to_outside() {
    let line    = (Coord2(5.0, 4.0), Coord2(20.0, 4.0));
    let bounds  = (Coord2(1.0, 1.0), Coord2(10.0, 10.0));
    let clipped = line_clip_to_bounds(&line, &bounds);

    assert!(clipped.is_some());

    let clipped = clipped.unwrap();
    assert!(clipped.0.distance_to(&Coord2(5.0, 4.0)) < 0.01);
    assert!(clipped.1.distance_to(&Coord2(10.0, 4.0)) < 0.01);
}

#[test]
fn horizontal_clipped_line_inside_to_outside_reverse() {
    let line    = (Coord2(20.0, 4.0), Coord2(5.0, 4.0));
    let bounds  = (Coord2(1.0, 1.0), Coord2(10.0, 10.0));
    let clipped = line_clip_to_bounds(&line, &bounds);

    assert!(clipped.is_some());

    let clipped = clipped.unwrap();
    assert!(clipped.0.distance_to(&Coord2(5.0, 4.0)) < 0.01);
    assert!(clipped.1.distance_to(&Coord2(10.0, 4.0)) < 0.01);
}

#[test]
fn vertical_clipped_line_inside_to_outside() {
    let line    = (Coord2(5.0, 4.0), Coord2(4.0, 20.0));
    let bounds  = (Coord2(1.0, 1.0), Coord2(10.0, 10.0));
    let clipped = line_clip_to_bounds(&line, &bounds);

    println!("{:?}", clipped);
    assert!(clipped.is_some());

    let clipped = clipped.unwrap();
    assert!(clipped.0.distance_to(&Coord2(5.0, 4.0)) < 0.01);
    assert!(clipped.1.distance_to(&Coord2(4.0, 10.0)) < 0.01);
}

#[test]
fn line_out_of_bounds() {
    let line    = (Coord2(0.0, 9.5), Coord2(20.0, 9.0));
    let bounds  = (Coord2(1.0, 1.0), Coord2(10.0, 10.0));
    let clipped = line_clip_to_bounds(&line, &bounds);

    println!("{:?}", clipped);

    assert!(clipped.is_none());
}
