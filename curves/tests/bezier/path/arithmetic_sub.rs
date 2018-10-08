use flo_curves::*;
use flo_curves::arc::*;
use flo_curves::bezier::path::*;

#[test]
fn subtract_circles() {
    // Two overlapping circles
    let circle1 = Circle::new(Coord2(5.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let circle2 = Circle::new(Coord2(7.0, 5.0), 4.0).to_path::<SimpleBezierPath>();

    // Combine them
    let combined_circles = path_sub::<_, _, _, SimpleBezierPath>(&vec![circle1], &vec![circle2], 0.01);

    assert!(combined_circles.len() == 1);

    // All points should be on either circle, and two should be on both
    let points = combined_circles[0].points().map(|(_, _, end_point)| end_point);

    let mut num_points_on_circle1   = 0;
    let mut num_points_on_circle2   = 0;
    let mut num_points_on_both      = 0;

    for point in points {
        let distance_to_circle1 = Coord2(5.0, 5.0).distance_to(&point);
        let distance_to_circle2 = Coord2(7.0, 5.0).distance_to(&point);

        // Must be on either circle
        assert!((distance_to_circle1-4.0).abs() < 0.01 || (distance_to_circle2-4.0).abs() < 0.01);

        println!("{:?} {:?} {:?}", point, distance_to_circle1, distance_to_circle2);

        if (distance_to_circle1-4.0).abs() < 0.01 && (distance_to_circle2-4.0).abs() < 0.01 { num_points_on_both += 1 }
        else if (distance_to_circle1-4.0).abs() < 0.01 { num_points_on_circle1 += 1 }
        else if (distance_to_circle2-4.0).abs() < 0.01 { num_points_on_circle2 += 1 }
    }

    println!("{:?} {:?} {:?}", num_points_on_circle1, num_points_on_circle2, num_points_on_both);

    assert!(num_points_on_circle1 == 2);
    assert!(num_points_on_circle2 == 2);
    assert!(num_points_on_both == 2);
}

#[test]
fn create_doughnut() {
    // Two overlapping circles
    let circle1 = Circle::new(Coord2(5.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let circle2 = Circle::new(Coord2(5.0, 5.0), 3.9).to_path::<SimpleBezierPath>();

    // Create a hole in the larger circle
    let combined_circles = path_sub::<_, _, _, SimpleBezierPath>(&vec![circle1], &vec![circle2], 0.01);

    assert!(combined_circles.len() == 2);
}

#[test]
fn erase_all() {
    // Two overlapping circles
    let circle1 = Circle::new(Coord2(5.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let circle2 = Circle::new(Coord2(5.0, 5.0), 3.9).to_path::<SimpleBezierPath>();

    // Create a hole in the larger circle
    let combined_circles = path_sub::<_, _, _, SimpleBezierPath>(&vec![circle2], &vec![circle1], 0.01);

    assert!(combined_circles.len() == 0);
}

#[test]
fn cut_corners() {
    // Two rectangles
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(4.0, 4.0))
        .line_to(Coord2(6.0, 4.0))
        .line_to(Coord2(6.0, 6.0))
        .line_to(Coord2(4.0, 6.0))
        .line_to(Coord2(4.0, 4.0))
        .build();

    // Subtract them
    let cut_corner = path_sub::<_, _, _, SimpleBezierPath>(&vec![rectangle1], &vec![rectangle2], 0.01);

    assert!(cut_corner.len() == 1);

    let cut_corner  = &cut_corner[0];
    let points      = cut_corner.points().collect::<Vec<_>>();

    assert!(cut_corner.start_point().distance_to(&Coord2(1.0, 1.0)) < 0.1);
    assert!(points[0].2.distance_to(&Coord2(5.0, 1.0)) < 0.1);
    assert!(points[1].2.distance_to(&Coord2(5.0, 4.0)) < 0.1);
    assert!(points[2].2.distance_to(&Coord2(4.0, 4.0)) < 0.1);
    assert!(points[3].2.distance_to(&Coord2(4.0, 5.0)) < 0.1);
    assert!(points[4].2.distance_to(&Coord2(1.0, 5.0)) < 0.1);
    assert!(points[5].2.distance_to(&Coord2(1.0, 1.0)) < 0.1);
}
