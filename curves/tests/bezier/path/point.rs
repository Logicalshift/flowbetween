use flo_curves::*;
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
    assert!(!path_contains_point(&path, &Coord2(-5.0, 5.0)));
    assert!(!path_contains_point(&path, &Coord2(3.0, 20.0)));
}
