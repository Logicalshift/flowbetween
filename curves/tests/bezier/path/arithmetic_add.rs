use flo_curves::*;
use flo_curves::arc::*;
use flo_curves::bezier::path::*;

#[test]
fn add_two_overlapping_circles() {
    // Two overlapping circles
    let circle1 = Circle::new(Coord2(5.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let circle2 = Circle::new(Coord2(7.0, 5.0), 4.0).to_path::<SimpleBezierPath>();

    // Combine them
    let combined_circles = path_add::<_, _, _, SimpleBezierPath>(&vec![circle1], &vec![circle2], 0.1);

    println!("{:?}", combined_circles);
    assert!(combined_circles.len() == 1);
}

/* -- should pass once we add support for mutliple paths
#[test]
fn add_two_non_overlapping_circles() {
    // Two overlapping circles
    let circle1 = Circle::new(Coord2(5.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let circle2 = Circle::new(Coord2(10.0, 5.0), 4.0).to_path::<SimpleBezierPath>();

    // Combine them
    let combined_circles = path_add::<_, _, _, SimpleBezierPath>(&vec![circle1], &vec![circle2], 0.1);

    println!("{:?}", combined_circles);
    assert!(combined_circles.len() == 2);
}
*/
