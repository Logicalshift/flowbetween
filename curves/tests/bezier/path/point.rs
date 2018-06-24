use flo_curves::*;
use flo_curves::arc::*;
use flo_curves::bezier::path::*;

#[test]
fn simple_path_contains_point() {
    // Path is a square
    let path = (Coord2(1.0, 2.0), vec![
        (Coord2(3.0, 2.0), Coord2(6.0, 2.0), Coord2(9.0, 2.0)), 
        (Coord2(9.0, 4.0), Coord2(9.0, 6.0), Coord2(9.0, 8.0)), 
        (Coord2(6.0, 8.0), Coord2(3.0, 8.0), Coord2(1.0, 8.0)),
        (Coord2(1.0, 6.0), Coord2(1.0, 4.0), Coord2(1.0, 2.0))
    ]);

    // Point should be inside
    assert!(path_contains_point(&path, &Coord2(5.0, 5.0)));
    assert!(path_contains_point(&path, &Coord2(3.0, 4.0)));
}

#[test]
fn circle_contains_point() {
    // Path is a circle
    let path: SimpleBezierPath = Circle::new(Coord2(5.0, 5.0), 4.0).to_path();

    // Point should be inside
    assert!(path_contains_point(&path, &Coord2(5.0, 5.0)));
    assert!(path_contains_point(&path, &Coord2(3.0, 4.0)));
    assert!(path_contains_point(&path, &Coord2(7.5, 7.5)));
    assert!(path_contains_point(&path, &Coord2(2.5, 7.5)));
}

#[test]
fn circle_edge_is_inside() {
    // Path is a circle
    let path: SimpleBezierPath = Circle::new(Coord2(5.0, 5.0), 4.0).to_path();

    // Points on the edge of the circle should be inside
    assert!(path_contains_point(&path, &Coord2(8.8, 5.0)));
    assert!(path_contains_point(&path, &Coord2(8.9, 5.0)));
    assert!(path_contains_point(&path, &Coord2(8.99, 5.0)));
    assert!(path_contains_point(&path, &Coord2(8.999, 5.0)));

    assert!(path_contains_point(&path, &Coord2(5.0, 8.8)));
    assert!(path_contains_point(&path, &Coord2(5.0, 8.9)));
    assert!(path_contains_point(&path, &Coord2(5.0, 8.99)));

    assert!(path_contains_point(&path, &Coord2(1.2, 5.0)));
    assert!(path_contains_point(&path, &Coord2(1.1, 5.0)));
    assert!(path_contains_point(&path, &Coord2(1.01, 5.0)));

    assert!(path_contains_point(&path, &Coord2(5.0, 1.2)));
    assert!(path_contains_point(&path, &Coord2(5.0, 1.1)));
    assert!(path_contains_point(&path, &Coord2(5.0, 1.01)));
}

#[test]
fn point_on_edge_is_in_path() {
    // Path is a square
    let path = (Coord2(1.0, 2.0), vec![
        (Coord2(3.0, 2.0), Coord2(6.0, 2.0), Coord2(9.0, 2.0)), 
        (Coord2(9.0, 4.0), Coord2(9.0, 6.0), Coord2(9.0, 8.0)), 
        (Coord2(6.0, 8.0), Coord2(3.0, 8.0), Coord2(1.0, 8.0)),
        (Coord2(1.0, 6.0), Coord2(1.0, 4.0), Coord2(1.0, 2.0))
    ]);

    // Points on the boundary should be inside
    assert!(path_contains_point(&path, &Coord2(5.0, 2.0)));
    assert!(path_contains_point(&path, &Coord2(1.0, 4.0)));
}

#[test]
fn points_outside_bounds_are_outside_path() {
    // Path is a square
    let path = (Coord2(1.0, 2.0), vec![
        (Coord2(3.0, 2.0), Coord2(6.0, 2.0), Coord2(9.0, 2.0)), 
        (Coord2(9.0, 4.0), Coord2(9.0, 6.0), Coord2(9.0, 8.0)), 
        (Coord2(6.0, 8.0), Coord2(3.0, 8.0), Coord2(1.0, 8.0)),
        (Coord2(1.0, 6.0), Coord2(1.0, 4.0), Coord2(1.0, 2.0))
    ]);

    // Points far outside the path should be outside
    assert!(!path_contains_point(&path, &Coord2(5.0, 20.0)));
    assert!(!path_contains_point(&path, &Coord2(5.0, -5.0)));
    assert!(!path_contains_point(&path, &Coord2(20.0, 5.0)));
    assert!(!path_contains_point(&path, &Coord2(-5.0, 5.0)));
    assert!(!path_contains_point(&path, &Coord2(3.0, 20.0)));
}

#[test]
fn circle_edges_do_not_contain_point() {
    // Path is a circle
    let path: SimpleBezierPath = Circle::new(Coord2(5.0, 5.0), 4.0).to_path();

    // Points should be inside the bounds but not in the circle
    assert!(!path_contains_point(&path, &Coord2(8.5, 8.5)));
    assert!(!path_contains_point(&path, &Coord2(1.5, 1.5)));
}
