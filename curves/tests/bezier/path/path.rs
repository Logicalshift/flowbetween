use flo_curves::*;
use flo_curves::bezier::path::*;

#[test]
fn reverse_rectangle() {
    let rectangle = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();

    let reversed = rectangle.reversed::<SimpleBezierPath>();

    assert!(reversed.start_point() == Coord2(1.0, 1.0));

    let points = reversed.points().collect::<Vec<_>>();
    assert!(points.len() == 4);
    assert!(points[0].2 == Coord2(5.0, 1.0));
    assert!(points[1].2 == Coord2(5.0, 5.0));
    assert!(points[2].2 == Coord2(1.0, 5.0));
    assert!(points[3].2 == Coord2(1.0, 1.0));
}
