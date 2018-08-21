extern crate flo_curves;

use flo_curves::*;

#[test]
fn overlapping_rects() {
    let r1 = (Coord2(30.0, 30.0), Coord2(60.0, 40.0));
    let r2 = (Coord2(20.0, 25.0), Coord2(35.0, 35.0));

    assert!(r1.overlaps(&r2));
}

#[test]
fn non_overlapping_rects() {
    let r1 = (Coord2(30.0, 30.0), Coord2(60.0, 40.0));
    let r2 = (Coord2(20.0, 25.0), Coord2(9.0, 10.0));

    assert!(!r1.overlaps(&r2));
}

#[test]
fn touching_rects() {
    let r1 = (Coord2(30.0, 30.0), Coord2(60.0, 40.0));
    let r2 = (Coord2(20.0, 25.0), Coord2(30.0, 30.0));

    assert!(!r1.overlaps(&r2));
}

#[test]
fn overlap_interior_rect() {
    let r1 = (Coord2(30.0, 30.0), Coord2(60.0, 50.0));
    let r2 = (Coord2(35.0, 35.0), Coord2(55.0, 45.0));

    assert!(r1.overlaps(&r2));
}

#[test]
fn overlap_exterior_rect() {
    let r1 = (Coord2(30.0, 30.0), Coord2(60.0, 40.0));
    let r2 = (Coord2(20.0, 20.0), Coord2(70.0, 50.0));

    assert!(r1.overlaps(&r2));
}

#[test]
fn from_points() {
    let r = Bounds::<Coord2>::bounds_for_points(vec![
        Coord2(30.0, 30.0),
        Coord2(60.0, 40.0),
        Coord2(45.0, 70.0),
        Coord2(10.0, 35.0)
    ]);

    assert!(r.min() == Coord2(10.0, 30.0));
    assert!(r.max() == Coord2(60.0, 70.00));
}
