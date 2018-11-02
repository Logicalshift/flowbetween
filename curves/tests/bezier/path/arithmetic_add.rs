use flo_curves::*;
use flo_curves::arc::*;
use flo_curves::bezier::path::*;

use super::svg::*;

#[test]
fn add_two_overlapping_circles() {
    // Two overlapping circles
    let circle1 = Circle::new(Coord2(5.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let circle2 = Circle::new(Coord2(7.0, 5.0), 4.0).to_path::<SimpleBezierPath>();

    // Combine them
    let combined_circles = path_add::<_, _, _, SimpleBezierPath>(&vec![circle1], &vec![circle2], 0.01);

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
fn add_circle_inside_circle() {
    // Two overlapping circles
    let circle1 = Circle::new(Coord2(5.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let circle2 = Circle::new(Coord2(5.0, 5.0), 3.9).to_path::<SimpleBezierPath>();

    // Combine them
    let combined_circles = path_add::<_, _, _, SimpleBezierPath>(&vec![circle1], &vec![circle2], 0.01);

    assert!(combined_circles.len() == 1);

    // All points should be on either circle, and two should be on both
    let points = combined_circles[0].points().map(|(_, _, end_point)| end_point);

    let mut num_points_on_circle1   = 0;

    for point in points {
        let distance_to_circle1 = Coord2(5.0, 5.0).distance_to(&point);

        // Must be on the circle
        assert!((distance_to_circle1-4.0).abs() < 0.01);
        if (distance_to_circle1-4.0).abs() < 0.01 { num_points_on_circle1 += 1 }
    }

    assert!(num_points_on_circle1 == 4);
}

#[test]
fn add_two_overlapping_circles_further_apart() {
    // Two overlapping circles
    let circle1 = Circle::new(Coord2(5.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let circle2 = Circle::new(Coord2(12.9, 5.0), 4.0).to_path::<SimpleBezierPath>();

    // Combine them
    let combined_circles = path_add::<_, _, _, SimpleBezierPath>(&vec![circle1], &vec![circle2], 0.01);

    assert!(combined_circles.len() == 1);

    // All points should be on either circle, and two should be on both
    let points = combined_circles[0].points().map(|(_, _, end_point)| end_point);

    let mut num_points_on_circle1   = 0;
    let mut num_points_on_circle2   = 0;
    let mut num_points_on_both      = 0;

    for point in points {
        let distance_to_circle1 = Coord2(5.0, 5.0).distance_to(&point);
        let distance_to_circle2 = Coord2(12.9, 5.0).distance_to(&point);

        // Must be on either circle
        assert!((distance_to_circle1-4.0).abs() < 0.01 || (distance_to_circle2-4.0).abs() < 0.01);

        println!("{:?} {:?} {:?}", point, distance_to_circle1, distance_to_circle2);

        if (distance_to_circle1-4.0).abs() < 0.01 && (distance_to_circle2-4.0).abs() < 0.01 { num_points_on_both += 1 }
        else if (distance_to_circle1-4.0).abs() < 0.01 { num_points_on_circle1 += 1 }
        else if (distance_to_circle2-4.0).abs() < 0.01 { num_points_on_circle2 += 1 }
    }

    println!("{:?} {:?} {:?}", num_points_on_circle1, num_points_on_circle2, num_points_on_both);

    assert!(num_points_on_circle1 == 4);
    assert!(num_points_on_circle2 == 4);
    assert!(num_points_on_both == 2);
}

#[test]
fn add_two_overlapping_circles_with_one_reversed() {
    // Two overlapping circles (one clockwise, one anti-clockwise)
    let circle1 = Circle::new(Coord2(5.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let circle2 = Circle::new(Coord2(7.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let circle2 = circle2.reversed::<SimpleBezierPath>();

    // Combine them
    let combined_circles = path_add::<_, _, _, SimpleBezierPath>(&vec![circle1], &vec![circle2], 0.01);

    println!("{:?}", combined_circles);
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
fn add_two_non_overlapping_circles() {
    // Two overlapping circles
    let circle1 = Circle::new(Coord2(5.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let circle2 = Circle::new(Coord2(20.0, 5.0), 4.0).to_path::<SimpleBezierPath>();

    // Combine them
    let combined_circles = path_add::<_, _, _, SimpleBezierPath>(&vec![circle1], &vec![circle2], 0.1);

    println!("{:?}", combined_circles);
    assert!(combined_circles.len() == 2);
}

#[test]
fn add_two_doughnuts() {
    // Two overlapping circles
    let circle1         = Circle::new(Coord2(5.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let inner_circle1   = Circle::new(Coord2(5.0, 5.0), 3.9).to_path::<SimpleBezierPath>();
    let circle2         = Circle::new(Coord2(9.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let inner_circle2   = Circle::new(Coord2(9.0, 5.0), 3.9).to_path::<SimpleBezierPath>();

    println!("{}", svg_path_string(&circle1));
    println!("{}", svg_path_string(&inner_circle1));
    println!("{}", svg_path_string(&circle2));
    println!("{}", svg_path_string(&inner_circle2));

    // Combine them
    let combined_circles = path_add::<_, _, _, SimpleBezierPath>(&vec![circle1, inner_circle1], &vec![circle2, inner_circle2], 0.1);

    println!("{:?}", combined_circles.len());
    println!("{:?}", combined_circles);
    assert!(combined_circles.len() == 4);
}

#[test]
fn remove_interior_points_basic() {
    let with_interior_point = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(2.0, 2.0))
        .line_to(Coord2(4.0, 2.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(1.0, 1.0))
        .build();

    let with_points_removed: Vec<SimpleBezierPath> = path_remove_interior_points(&vec![with_interior_point], 0.1);

    // Should be 5 points in the path with points removed
    assert!(with_points_removed.len() == 1);
    assert!(with_points_removed[0].points().count() != 6);
    assert!(with_points_removed[0].points().count() == 5);

    let expected_points = vec![
        Coord2(1.0, 1.0),
        Coord2(1.0, 5.0),
        Coord2(5.0, 5.0),
        Coord2(5.0, 1.0),
        Coord2(3.0, 3.0)
    ];

    assert!(expected_points.iter().any(|expected| with_points_removed[0].start_point().distance_to(expected) < 0.1));
    for (_cp1, _cp2, point) in with_points_removed[0].points() {
        assert!(expected_points.iter().any(|expected| point.distance_to(expected) < 0.1));
    }
}

#[test]
fn rectangle_add_graph_path() {
    // Two rectangles
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(3.0, 3.0))
        .line_to(Coord2(7.0, 3.0))
        .line_to(Coord2(7.0, 7.0))
        .line_to(Coord2(3.0, 7.0))
        .line_to(Coord2(3.0, 3.0))
        .build();

    let path = GraphPath::from_path(&rectangle1, ());
    assert!(path.all_edges().count() == 4);
    let path = path.collide(GraphPath::from_path(&rectangle2, ()), 0.01);
    assert!(path.all_edges().count() == 12);
}

#[test]
fn rectangle_add() {
    // Two rectangles
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(3.0, 3.0))
        .line_to(Coord2(7.0, 3.0))
        .line_to(Coord2(7.0, 7.0))
        .line_to(Coord2(3.0, 7.0))
        .line_to(Coord2(3.0, 3.0))
        .build();

    // Add them
    let shared_point = path_add::<_, _, _, SimpleBezierPath>(&vec![rectangle1], &vec![rectangle2], 0.01);

    assert!(shared_point.len() == 1);

    let shared_point    = &shared_point[0];
    let points          = shared_point.points().collect::<Vec<_>>();

    assert!(shared_point.start_point().distance_to(&Coord2(1.0, 1.0)) < 0.1);
    assert!(points[0].2.distance_to(&Coord2(5.0, 1.0)) < 0.1);
    assert!(points[1].2.distance_to(&Coord2(5.0, 3.0)) < 0.1);
    assert!(points[2].2.distance_to(&Coord2(7.0, 3.0)) < 0.1);
    assert!(points[3].2.distance_to(&Coord2(7.0, 7.0)) < 0.1);
    assert!(points[4].2.distance_to(&Coord2(3.0, 7.0)) < 0.1);
    assert!(points[5].2.distance_to(&Coord2(3.0, 5.0)) < 0.1);
    assert!(points[6].2.distance_to(&Coord2(1.0, 5.0)) < 0.1);
    assert!(points[7].2.distance_to(&Coord2(1.0, 1.0)) < 0.1);
}

#[test]
fn rectangle_add_with_shared_point() {
    // Two rectangles
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(5.0, 3.0)) // Shared point
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(3.0, 3.0))
        .line_to(Coord2(5.0, 3.0)) // Shared point
        .line_to(Coord2(7.0, 3.0))
        .line_to(Coord2(7.0, 7.0))
        .line_to(Coord2(3.0, 7.0))
        .line_to(Coord2(3.0, 3.0))
        .build();

    // Add them
    let shared_point = path_add::<_, _, _, SimpleBezierPath>(&vec![rectangle1], &vec![rectangle2], 0.01);

    assert!(shared_point.len() == 1);

    let shared_point    = &shared_point[0];
    let points          = shared_point.points().collect::<Vec<_>>();

    assert!(shared_point.start_point().distance_to(&Coord2(1.0, 1.0)) < 0.1);
    assert!(points[0].2.distance_to(&Coord2(5.0, 1.0)) < 0.1);
    assert!(points[1].2.distance_to(&Coord2(5.0, 3.0)) < 0.1);
    assert!(points[2].2.distance_to(&Coord2(7.0, 3.0)) < 0.1);
    assert!(points[3].2.distance_to(&Coord2(7.0, 7.0)) < 0.1);
    assert!(points[4].2.distance_to(&Coord2(3.0, 7.0)) < 0.1);
    assert!(points[5].2.distance_to(&Coord2(3.0, 5.0)) < 0.1);
    assert!(points[6].2.distance_to(&Coord2(1.0, 5.0)) < 0.1);
    assert!(points[7].2.distance_to(&Coord2(1.0, 1.0)) < 0.1);
}

#[test]
fn rectangle_add_with_shared_point_2() {
    // Two rectangles
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(3.0, 5.0)) // Shared point
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(3.0, 3.0))
        .line_to(Coord2(7.0, 3.0))
        .line_to(Coord2(7.0, 7.0))
        .line_to(Coord2(3.0, 7.0))
        .line_to(Coord2(3.0, 5.0)) // Shared point
        .line_to(Coord2(3.0, 3.0))
        .build();

    // Add them
    let shared_point = path_add::<_, _, _, SimpleBezierPath>(&vec![rectangle1], &vec![rectangle2], 0.01);

    assert!(shared_point.len() == 1);

    let shared_point    = &shared_point[0];
    let points          = shared_point.points().collect::<Vec<_>>();

    assert!(shared_point.start_point().distance_to(&Coord2(1.0, 1.0)) < 0.1);
    assert!(points[0].2.distance_to(&Coord2(5.0, 1.0)) < 0.1);
    assert!(points[1].2.distance_to(&Coord2(5.0, 3.0)) < 0.1);
    assert!(points[2].2.distance_to(&Coord2(7.0, 3.0)) < 0.1);
    assert!(points[3].2.distance_to(&Coord2(7.0, 7.0)) < 0.1);
    assert!(points[4].2.distance_to(&Coord2(3.0, 7.0)) < 0.1);
    assert!(points[5].2.distance_to(&Coord2(3.0, 5.0)) < 0.1);
    assert!(points[6].2.distance_to(&Coord2(1.0, 5.0)) < 0.1);
    assert!(points[7].2.distance_to(&Coord2(1.0, 1.0)) < 0.1);
}

#[test]
fn rectangle_add_with_shared_point_3() {
    // Two rectangles
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(5.0, 3.0)) // Shared point
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(3.0, 5.0)) // Shared point
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(3.0, 3.0))
        .line_to(Coord2(5.0, 3.0)) // Shared point
        .line_to(Coord2(7.0, 3.0))
        .line_to(Coord2(7.0, 7.0))
        .line_to(Coord2(3.0, 7.0))
        .line_to(Coord2(3.0, 5.0)) // Shared point
        .line_to(Coord2(3.0, 3.0))
        .build();

    // Add them
    let shared_point = path_add::<_, _, _, SimpleBezierPath>(&vec![rectangle1], &vec![rectangle2], 0.01);

    assert!(shared_point.len() == 1);

    let shared_point    = &shared_point[0];
    let points          = shared_point.points().collect::<Vec<_>>();

    assert!(shared_point.start_point().distance_to(&Coord2(1.0, 1.0)) < 0.1);
    assert!(points[0].2.distance_to(&Coord2(5.0, 1.0)) < 0.1);
    assert!(points[1].2.distance_to(&Coord2(5.0, 3.0)) < 0.1);
    assert!(points[2].2.distance_to(&Coord2(7.0, 3.0)) < 0.1);
    assert!(points[3].2.distance_to(&Coord2(7.0, 7.0)) < 0.1);
    assert!(points[4].2.distance_to(&Coord2(3.0, 7.0)) < 0.1);
    assert!(points[5].2.distance_to(&Coord2(3.0, 5.0)) < 0.1);
    assert!(points[6].2.distance_to(&Coord2(1.0, 5.0)) < 0.1);
    assert!(points[7].2.distance_to(&Coord2(1.0, 1.0)) < 0.1);
}

#[test]
fn rectangle_add_with_shared_point_4() {
    // Two rectangles
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(5.0, 3.0)) // Shared point
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(3.0, 5.0)) // Shared point
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(3.0, 3.0))
        .line_to(Coord2(5.0, 3.0)) // Shared point
        .line_to(Coord2(7.0, 3.0))
        .line_to(Coord2(7.0, 7.0))
        .line_to(Coord2(3.0, 7.0))
        .line_to(Coord2(3.0, 5.0)) // Shared point
        .line_to(Coord2(3.0, 3.0))
        .build()
        .reversed::<SimpleBezierPath>();

    // Print out the graph path generated by adding these two points
    let mut gp = GraphPath::from_path(&rectangle1, PathLabel(PathSource::Path1, PathDirection::Clockwise)).collide(GraphPath::from_path(&rectangle2, PathLabel(PathSource::Path2, PathDirection::Clockwise)), 0.01);
    gp.set_exterior_by_adding();
    println!("{:?}", gp);

    // Add them
    let shared_point = path_add::<_, _, _, SimpleBezierPath>(&vec![rectangle1], &vec![rectangle2], 0.01);

    assert!(shared_point.len() == 1);

    let shared_point    = &shared_point[0];
    let points          = shared_point.points().collect::<Vec<_>>();

    assert!(shared_point.start_point().distance_to(&Coord2(1.0, 1.0)) < 0.1);
    assert!(points[0].2.distance_to(&Coord2(5.0, 1.0)) < 0.1);
    assert!(points[1].2.distance_to(&Coord2(5.0, 3.0)) < 0.1);
    assert!(points[2].2.distance_to(&Coord2(7.0, 3.0)) < 0.1);
    assert!(points[3].2.distance_to(&Coord2(7.0, 7.0)) < 0.1);
    assert!(points[4].2.distance_to(&Coord2(3.0, 7.0)) < 0.1);
    assert!(points[5].2.distance_to(&Coord2(3.0, 5.0)) < 0.1);
    assert!(points[6].2.distance_to(&Coord2(1.0, 5.0)) < 0.1);
    assert!(points[7].2.distance_to(&Coord2(1.0, 1.0)) < 0.1);
}

#[test]
fn rectangle_add_with_shared_point_5() {
    // Two rectangles
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(5.0, 3.0)) // Shared point
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(3.0, 5.0)) // Shared point
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(1.0, 1.0))
        .build()
        .reversed::<SimpleBezierPath>();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(3.0, 3.0))
        .line_to(Coord2(5.0, 3.0)) // Shared point
        .line_to(Coord2(7.0, 3.0))
        .line_to(Coord2(7.0, 7.0))
        .line_to(Coord2(3.0, 7.0))
        .line_to(Coord2(3.0, 5.0)) // Shared point
        .line_to(Coord2(3.0, 3.0))
        .build();

    // Add them
    let shared_point = path_add::<_, _, _, SimpleBezierPath>(&vec![rectangle1], &vec![rectangle2], 0.01);

    assert!(shared_point.len() == 1);

    let shared_point    = &shared_point[0];
    let points          = shared_point.points().collect::<Vec<_>>();

    assert!(shared_point.start_point().distance_to(&Coord2(1.0, 1.0)) < 0.1);
    assert!(points[0].2.distance_to(&Coord2(1.0, 5.0)) < 0.1);
    assert!(points[1].2.distance_to(&Coord2(3.0, 5.0)) < 0.1);
    assert!(points[2].2.distance_to(&Coord2(3.0, 7.0)) < 0.1);
    assert!(points[3].2.distance_to(&Coord2(7.0, 7.0)) < 0.1);
    assert!(points[4].2.distance_to(&Coord2(7.0, 3.0)) < 0.1);
    assert!(points[5].2.distance_to(&Coord2(5.0, 3.0)) < 0.1);
    assert!(points[6].2.distance_to(&Coord2(5.0, 1.0)) < 0.1);
    assert!(points[7].2.distance_to(&Coord2(1.0, 1.0)) < 0.1);
}

#[test]
fn rectangle_add_with_shared_point_6() {
    // Two rectangles
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(5.0, 3.0)) // Shared point
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(3.0, 5.0)) // Shared point
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(1.0, 1.0))
        .build()
        .reversed::<SimpleBezierPath>();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(3.0, 3.0))
        .line_to(Coord2(7.0, 3.0))
        .line_to(Coord2(7.0, 7.0))
        .line_to(Coord2(3.0, 7.0))
        .line_to(Coord2(3.0, 3.0))
        .build();

    // Add them
    let shared_point = path_add::<_, _, _, SimpleBezierPath>(&vec![rectangle1], &vec![rectangle2], 0.01);

    assert!(shared_point.len() == 1);

    let shared_point    = &shared_point[0];
    let points          = shared_point.points().collect::<Vec<_>>();

    assert!(shared_point.start_point().distance_to(&Coord2(1.0, 1.0)) < 0.1);
    assert!(points[0].2.distance_to(&Coord2(1.0, 5.0)) < 0.1);
    assert!(points[1].2.distance_to(&Coord2(3.0, 5.0)) < 0.1);
    assert!(points[2].2.distance_to(&Coord2(3.0, 7.0)) < 0.1);
    assert!(points[3].2.distance_to(&Coord2(7.0, 7.0)) < 0.1);
    assert!(points[4].2.distance_to(&Coord2(7.0, 3.0)) < 0.1);
    assert!(points[5].2.distance_to(&Coord2(5.0, 3.0)) < 0.1);
    assert!(points[6].2.distance_to(&Coord2(5.0, 1.0)) < 0.1);
    assert!(points[7].2.distance_to(&Coord2(1.0, 1.0)) < 0.1);
}
