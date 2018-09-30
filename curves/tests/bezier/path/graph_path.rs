use flo_curves::*;
use flo_curves::arc::*;
use flo_curves::bezier::path::*;

#[test]
pub fn create_and_read_simple_graph_path() {
    let path            = (Coord2(10.0, 11.0), vec![(Coord2(15.0, 16.0), Coord2(17.0, 18.0), Coord2(19.0, 20.0)), (Coord2(21.0, 22.0), Coord2(23.0, 24.0), Coord2(25.0, 26.0))]);
    let graph_path      = GraphPath::from_path(&path, ());

    assert!(graph_path.num_points() == 3);

    // Point 0 edges
    {
        let edges = graph_path.edges_for_point(0).collect::<Vec<_>>();

        assert!(edges.len() == 1);
        assert!(edges[0].kind() == GraphPathEdgeKind::Uncategorised);
        assert!(edges[0].start_point() == Coord2(10.0, 11.0));
        assert!(edges[0].end_point() == Coord2(19.0, 20.0));
        assert!(edges[0].control_points() == (Coord2(15.0, 16.0), Coord2(17.0, 18.0)));
    }

    // Point 1 edges
    {
        let edges = graph_path.edges_for_point(1).collect::<Vec<_>>();

        assert!(edges.len() == 1);
        assert!(edges[0].kind() == GraphPathEdgeKind::Uncategorised);
        assert!(edges[0].start_point() == Coord2(19.0, 20.0));
        assert!(edges[0].end_point() == Coord2(25.0, 26.0));
        assert!(edges[0].control_points() == (Coord2(21.0, 22.0), Coord2(23.0, 24.0)));
    }

    // Point 2 edges
    {
        let edges = graph_path.edges_for_point(2).collect::<Vec<_>>();
        assert!(edges.len() == 1);
        assert!(edges[0].kind() == GraphPathEdgeKind::Uncategorised);
        assert!(edges[0].start_point() == Coord2(25.0, 26.0));
        assert!(edges[0].end_point() == Coord2(10.0, 11.0));
    }
}

#[test]
pub fn create_and_read_simple_graph_path_reverse() {
    let path            = (Coord2(10.0, 11.0), vec![(Coord2(15.0, 16.0), Coord2(17.0, 18.0), Coord2(19.0, 20.0)), (Coord2(21.0, 22.0), Coord2(23.0, 24.0), Coord2(25.0, 26.0))]);
    let graph_path      = GraphPath::from_path(&path, ());

    assert!(graph_path.num_points() == 3);

    // Point 0 edges
    {
        let edges = graph_path.reverse_edges_for_point(0).collect::<Vec<_>>();

        assert!(edges.len() == 1);
        assert!(edges[0].kind() == GraphPathEdgeKind::Uncategorised);
        assert!(edges[0].start_point() == Coord2(10.0, 11.0));
        assert!(edges[0].end_point() == Coord2(25.0, 26.0));
        assert!(edges[0].control_points() == (Coord2(19.9, 20.9), Coord2(14.95, 15.95)));
    }

    // Point 1 edges
    {
        let edges = graph_path.reverse_edges_for_point(1).collect::<Vec<_>>();

        assert!(edges.len() == 1);
        assert!(edges[0].kind() == GraphPathEdgeKind::Uncategorised);
        assert!(edges[0].start_point() == Coord2(19.0, 20.0));
        assert!(edges[0].end_point() == Coord2(10.0, 11.0));
        assert!(edges[0].control_points() == (Coord2(17.0, 18.0), Coord2(15.0, 16.0)));
    }

    // Point 2 edges
    {
        let edges = graph_path.reverse_edges_for_point(2).collect::<Vec<_>>();
        assert!(edges.len() == 1);
        assert!(edges[0].kind() == GraphPathEdgeKind::Uncategorised);
        assert!(edges[0].start_point() == Coord2(25.0, 26.0));
        assert!(edges[0].end_point() == Coord2(19.0, 20.0));
        assert!(edges[0].control_points() == (Coord2(23.0, 24.0), Coord2(21.0, 22.0)));
    }
}

#[test]
pub fn collide_two_rectangles() {
    // Create the two rectangles
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(4.0, 4.0))
        .line_to(Coord2(9.0, 4.0))
        .line_to(Coord2(9.0, 9.0))
        .line_to(Coord2(4.0, 9.0))
        .line_to(Coord2(4.0, 4.0))
        .build();
    
    let rectangle1 = GraphPath::from_path(&rectangle1, 1);
    let rectangle2 = GraphPath::from_path(&rectangle2, 2);

    // Collide them
    let collision = rectangle1.collide(rectangle2, 0.1);

    // 10 points in the collision
    assert!(collision.num_points() == 10);

    let mut check_count = 0;

    for point_idx in 0..10 {
        // Check the edges for each point
        let edges = collision.edges_for_point(point_idx).collect::<Vec<_>>();

        assert!(edges.len() <= 2);
        assert!(edges.len() >= 1);

        assert!(edges[0].kind() == GraphPathEdgeKind::Uncategorised);
        assert!(edges.len() == 1 || edges[1].kind() == GraphPathEdgeKind::Uncategorised);

        // Edges leading up to the collision
        if edges[0].start_point() == Coord2(5.0, 1.0) {
            check_count += 1;

            assert!(edges.len() == 1);
            assert!(edges[0].end_point().distance_to(&Coord2(5.0, 4.0)) < 0.1);
            assert!(edges.iter().all(|edge| edge.label() == 1));
        }

        if edges[0].start_point() == Coord2(5.0, 5.0) {
            check_count += 1;

            assert!(edges.len() == 1);
            assert!(edges[0].end_point().distance_to(&Coord2(4.0, 5.0)) < 0.1);
            assert!(edges.iter().all(|edge| edge.label() == 1));
        }

        if edges[0].start_point() == Coord2(1.0, 5.0) {
            check_count += 1;

            assert!(edges.len() == 1);
            assert!(edges[0].end_point().distance_to(&Coord2(1.0, 1.0)) < 0.1);
            assert!(edges.iter().all(|edge| edge.label() == 1));
        }

        if edges[0].start_point() == Coord2(4.0, 4.0) {
            check_count += 1;

            assert!(edges.len() == 1);
            assert!(edges[0].end_point().distance_to(&Coord2(5.0, 4.0)) < 0.1);
            assert!(edges.iter().all(|edge| edge.label() == 2));
        }

        // Collision edges
        if edges[0].start_point().distance_to(&Coord2(4.0, 5.0)) < 0.1 {
            check_count += 1;

            assert!(edges.len() == 2);
            assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 4.0)) < 0.1));
            assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(1.0, 5.0)) < 0.1));
            assert!(edges.iter().any(|edge| edge.label() == 1));
            assert!(edges.iter().any(|edge| edge.label() == 2));
        }

        if edges[0].start_point().distance_to(&Coord2(5.0, 4.0)) < 0.1 {
            check_count += 1;

            assert!(edges.len() == 2);
            assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(9.0, 4.0)) < 0.1));
            assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(5.0, 5.0)) < 0.1));
            assert!(edges.iter().any(|edge| edge.label() == 1));
            assert!(edges.iter().any(|edge| edge.label() == 2));
        }
    }

    // Checked 6 (of 10) edges
    assert!(check_count == 6);
}

#[test]
fn multiple_collisions_on_one_edge() {
    // Create the two rectangles
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(2.0, 0.0))
        .line_to(Coord2(2.0, 6.0))
        .line_to(Coord2(4.0, 6.0))
        .line_to(Coord2(4.0, 0.0))
        .line_to(Coord2(2.0, 0.0))
        .build();
    
    let rectangle1 = GraphPath::from_path(&rectangle1, ());
    let rectangle2 = GraphPath::from_path(&rectangle2, ());

    // Collide them
    let collision = rectangle1.collide(rectangle2, 0.1);

    // 12 points in the collision
    assert!(collision.num_points() == 12);

    // Check the intersection points
    for point_idx in 0..12 {
        let edges = collision.edges_for_point(point_idx).collect::<Vec<_>>();

        assert!(edges.len() <= 2);
        if edges.len() == 2 {
            if edges[0].start_point().distance_to(&Coord2(2.0, 1.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 5.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(1.0, 1.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(4.0, 1.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 1.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 0.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(2.0, 5.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 6.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 5.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(4.0, 5.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(5.0, 5.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 1.0)) < 0.1));
            } else {
                // These are the only four intersection points that should exist
                println!("{:?}", edges[0].start_point());
                assert!(false)
            }
        }
    }
}

#[test]
fn multiple_collisions_on_one_edge_opposite_direction() {
    // Create the two rectangles
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(4.0, 0.0))
        .line_to(Coord2(4.0, 6.0))
        .line_to(Coord2(2.0, 6.0))
        .line_to(Coord2(2.0, 0.0))
        .line_to(Coord2(4.0, 0.0))
        .build();
    
    let rectangle1 = GraphPath::from_path(&rectangle1, ());
    let rectangle2 = GraphPath::from_path(&rectangle2, ());

    // Collide them
    let collision = rectangle1.collide(rectangle2, 0.1);

    // 12 points in the collision
    assert!(collision.num_points() == 12);

    // Check the intersection points
    let mut num_intersects = 0;
    for point_idx in 0..12 {
        let edges = collision.edges_for_point(point_idx).collect::<Vec<_>>();

        assert!(edges.len() <= 2);
        assert!(edges.len() > 0);
        if edges.len() == 2 {
            num_intersects += 1;

            if edges[0].start_point().distance_to(&Coord2(2.0, 1.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 0.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(1.0, 1.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(4.0, 1.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 1.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 5.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(2.0, 5.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 1.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 5.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(4.0, 5.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(5.0, 5.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 6.0)) < 0.1));
            } else {
                // These are the only four intersection points that should exist
                println!("{:?}", edges[0].start_point());
                assert!(false)
            }
        } else if edges.len() == 1 {
            let edge        = edges.iter().nth(0).unwrap();
            let start_point = edge.start_point();

            assert!((start_point.x()-1.0).abs() < 0.01 ||
                    (start_point.x()-5.0).abs() < 0.01 ||
                    (start_point.x()-2.0).abs() < 0.01 ||
                    (start_point.x()-4.0).abs() < 0.01);
            assert!((start_point.y()-1.0).abs() < 0.01 ||
                    (start_point.y()-5.0).abs() < 0.01 ||
                    (start_point.y()-0.0).abs() < 0.01 ||
                    (start_point.y()-6.0).abs() < 0.01);
        }
    }

    assert!(num_intersects == 4);
}

#[test]
fn collision_at_same_point() {
    // Two rectangles, with the collision point already subdivided
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(2.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(4.0, 0.0))
        .line_to(Coord2(4.0, 6.0))
        .line_to(Coord2(2.0, 6.0))
        .line_to(Coord2(2.0, 1.0))
        .line_to(Coord2(2.0, 0.0))
        .line_to(Coord2(4.0, 0.0))
        .build();
    
    let rectangle1 = GraphPath::from_path(&rectangle1, ());
    let rectangle2 = GraphPath::from_path(&rectangle2, ());

    // Collide them
    // TODO: find out why setting accuracy to 0.01 here produces only 10 points in the collision
    let collision = rectangle1.collide(rectangle2, 0.05);

    // 12 points in the collision (but we can allow for the shared point to be left as 'orphaned')
    assert!(collision.num_points() == 12 || collision.num_points() == 13);

    // If there are 13 points, one should have no edges any more (as another should have been chosen as the shared point)
    if collision.num_points() == 13 {
        let mut num_orphaned_points = 0;
        for point_idx in 0..13 {
            let edges = collision.edges_for_point(point_idx).collect::<Vec<_>>();
            if edges.len() == 0 { num_orphaned_points += 1; }
        }

        assert!(num_orphaned_points <= 1);
    }

    // Check the intersection points
    let mut num_intersects = 0;
    for point_idx in 0..collision.num_points() {
        let edges = collision.edges_for_point(point_idx).collect::<Vec<_>>();

        if edges.len() == 2 {
            num_intersects += 1;

            if edges[0].start_point().distance_to(&Coord2(2.0, 1.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 0.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(1.0, 1.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(4.0, 1.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 1.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 5.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(2.0, 5.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 1.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 5.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(4.0, 5.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(5.0, 5.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 6.0)) < 0.1));
            } else {
                // These are the only four intersection points that should exist
                println!("{:?}", edges[0].start_point());
                assert!(false)
            }
        } else if edges.len() == 1 {
            let edge        = edges.iter().nth(0).unwrap();
            let start_point = edge.start_point();

            assert!((start_point.x()-1.0).abs() < 0.01 ||
                    (start_point.x()-5.0).abs() < 0.01 ||
                    (start_point.x()-2.0).abs() < 0.01 ||
                    (start_point.x()-4.0).abs() < 0.01);
            assert!((start_point.y()-1.0).abs() < 0.01 ||
                    (start_point.y()-5.0).abs() < 0.01 ||
                    (start_point.y()-0.0).abs() < 0.01 ||
                    (start_point.y()-6.0).abs() < 0.01);
        } else {
            // Should only be 1 edge (corners) or 2 edges (collision points)
            // TODO: currently we end up here because we're generating point -> same point 'edges' for some reason
            println!("{:?}", edges);
            assert!(edges.len() <= 2);
        }
    }

    assert!(num_intersects == 4);
}

#[test]
fn collision_exactly_on_edge_src() {
    // Two rectangles, with the collision point already subdivided
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(2.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(4.0, 0.0))
        .line_to(Coord2(4.0, 6.0))
        .line_to(Coord2(2.0, 6.0))
        .line_to(Coord2(2.0, 0.0))
        .line_to(Coord2(4.0, 0.0))
        .build();
    
    let rectangle1 = GraphPath::from_path(&rectangle1, ());
    let rectangle2 = GraphPath::from_path(&rectangle2, ());

    // Collide them
    // TODO: find out why setting accuracy to 0.01 here produces only 10 points in the collision (hm, seems to be a limitation of the precision of the algorithm)
    // (turns out to be precision: setting the accuracy too high causes the subdivisions to never collide)
    let collision = rectangle1.collide(rectangle2, 0.05);

    // 12 points in the collision (but we can allow for the shared point to be left as 'orphaned')
    assert!(collision.num_points() == 12 || collision.num_points() == 13);

    // If there are 13 points, one should have no edges any more (as another should have been chosen as the shared point)
    if collision.num_points() == 13 {
        let mut num_orphaned_points = 0;
        for point_idx in 0..13 {
            let edges = collision.edges_for_point(point_idx).collect::<Vec<_>>();
            if edges.len() == 0 { num_orphaned_points += 1; }
        }

        assert!(num_orphaned_points <= 1);
    }

    // Check the intersection points
    let mut num_intersects = 0;
    for point_idx in 0..collision.num_points() {
        let edges = collision.edges_for_point(point_idx).collect::<Vec<_>>();

        if edges.len() == 2 {
            num_intersects += 1;

            if edges[0].start_point().distance_to(&Coord2(2.0, 1.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 0.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(1.0, 1.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(4.0, 1.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 1.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 5.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(2.0, 5.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 1.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 5.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(4.0, 5.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(5.0, 5.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 6.0)) < 0.1));
            } else {
                // These are the only four intersection points that should exist
                println!("{:?}", edges[0].start_point());
                assert!(false)
            }
        } else if edges.len() == 1 {
            let edge        = edges.iter().nth(0).unwrap();
            let start_point = edge.start_point();

            assert!((start_point.x()-1.0).abs() < 0.01 ||
                    (start_point.x()-5.0).abs() < 0.01 ||
                    (start_point.x()-2.0).abs() < 0.01 ||
                    (start_point.x()-4.0).abs() < 0.01);
            assert!((start_point.y()-1.0).abs() < 0.01 ||
                    (start_point.y()-5.0).abs() < 0.01 ||
                    (start_point.y()-0.0).abs() < 0.01 ||
                    (start_point.y()-6.0).abs() < 0.01);
        } else {
            // Should only be 1 edge (corners) or 2 edges (collision points)
            // TODO: currently we end up here because we're generating point -> same point 'edges' for some reason
            println!("{:?}", edges);
            assert!(edges.len() <= 2);
        }
    }

    assert!(num_intersects == 4);
}

#[test]
fn collision_exactly_on_edge_tgt() {
    // Two rectangles, with the collision point already subdivided
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(4.0, 0.0))
        .line_to(Coord2(4.0, 6.0))
        .line_to(Coord2(2.0, 6.0))
        .line_to(Coord2(2.0, 1.0))
        .line_to(Coord2(2.0, 0.0))
        .line_to(Coord2(4.0, 0.0))
        .build();
    
    let rectangle1 = GraphPath::from_path(&rectangle1, ());
    let rectangle2 = GraphPath::from_path(&rectangle2, ());

    // Collide them
    // TODO: find out why setting accuracy to 0.01 here produces only 10 points in the collision
    // (turns out to be precision: setting the accuracy too high causes the subdivisions to never collide)
    let collision = rectangle1.collide(rectangle2, 0.02);

    // 12 points in the collision (but we can allow for the shared point to be left as 'orphaned')
    assert!(collision.num_points() == 12 || collision.num_points() == 13);

    // If there are 13 points, one should have no edges any more (as another should have been chosen as the shared point)
    if collision.num_points() == 13 {
        let mut num_orphaned_points = 0;
        for point_idx in 0..13 {
            let edges = collision.edges_for_point(point_idx).collect::<Vec<_>>();
            if edges.len() == 0 { num_orphaned_points += 1; }
        }

        assert!(num_orphaned_points <= 1);
    }

    // Check the intersection points
    let mut num_intersects = 0;
    for point_idx in 0..collision.num_points() {
        let edges = collision.edges_for_point(point_idx).collect::<Vec<_>>();

        if edges.len() == 2 {
            num_intersects += 1;

            if edges[0].start_point().distance_to(&Coord2(2.0, 1.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 0.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(1.0, 1.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(4.0, 1.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 1.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 5.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(2.0, 5.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(2.0, 1.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 5.0)) < 0.1));
            } else if edges[0].start_point().distance_to(&Coord2(4.0, 5.0)) < 0.1 {
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(5.0, 5.0)) < 0.1));
                assert!(edges.iter().any(|edge| edge.end_point().distance_to(&Coord2(4.0, 6.0)) < 0.1));
            } else {
                // These are the only four intersection points that should exist
                println!("{:?}", edges[0].start_point());
                assert!(false)
            }
        } else if edges.len() == 1 {
            let edge        = edges.iter().nth(0).unwrap();
            let start_point = edge.start_point();

            assert!((start_point.x()-1.0).abs() < 0.01 ||
                    (start_point.x()-5.0).abs() < 0.01 ||
                    (start_point.x()-2.0).abs() < 0.01 ||
                    (start_point.x()-4.0).abs() < 0.01);
            assert!((start_point.y()-1.0).abs() < 0.01 ||
                    (start_point.y()-5.0).abs() < 0.01 ||
                    (start_point.y()-0.0).abs() < 0.01 ||
                    (start_point.y()-6.0).abs() < 0.01);
        } else {
            // Should only be 1 edge (corners) or 2 edges (collision points)
            // TODO: currently we end up here because we're generating point -> same point 'edges' for some reason
            println!("{:?}", edges);
            assert!(edges.len() <= 2);
        }
    }

    assert!(num_intersects == 4);
}

#[test]
fn cast_ray_to_rectangle_corner() {
    // Create a rectangle
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle1 = GraphPath::from_path(&rectangle1, ());

    // Collide against the top-left corner
    let collision = rectangle1.line_collision(&(Coord2(0.0, 0.0), Coord2(1.0, 1.0)));

    assert!(collision.is_some());

    let collision = collision.unwrap();
    assert!(collision.0.start_point() == Coord2(1.0, 1.0));
    assert!((collision.1-0.0).abs() < 0.01);
}

#[test]
fn cast_ray_across_rectangle() {
    // Create a rectangle
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle1 = GraphPath::from_path(&rectangle1, ());

    // Collide across the center of the rectangle
    let collision = rectangle1.line_collision(&(Coord2(0.0, 3.0), Coord2(6.0, 3.0)));

    assert!(collision.is_some());

    let collision = collision.unwrap();
    assert!(collision.0.point_at_pos(collision.1).distance_to(&Coord2(1.0, 3.0)) < 0.001);
    assert!(collision.0.start_point() == Coord2(1.0, 1.0));
    assert!((collision.1-0.5).abs() < 0.01);
}

#[test]
fn cast_ray_to_rectangle_far_corner() {
    // Create a rectangle
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle1 = GraphPath::from_path(&rectangle1, ());

    // Collide against all corners
    let collision = rectangle1.line_collision(&(Coord2(0.0, 0.0), Coord2(6.0, 6.0)));

    assert!(collision.is_some());

    let collision = collision.unwrap();
    assert!(collision.0.start_point() == Coord2(1.0, 1.0));
    assert!((collision.1-0.0).abs() < 0.01);
}

#[test]
fn cast_ray_to_rectangle_far_corner_backwards() {
    // Create a rectangle
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle1 = GraphPath::from_path(&rectangle1, ());

    // Collide against all corners
    let collision = rectangle1.line_collision(&(Coord2(6.0, 6.0), Coord2(0.0, 0.0)));

    assert!(collision.is_some());

    let collision = collision.unwrap();
    assert!(collision.0.start_point() == Coord2(1.0, 5.0));
    assert!((collision.1-1.0).abs() < 0.01);
}

#[test]
fn cast_ray_to_nowhere() {
    // Create a rectangle
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle1 = GraphPath::from_path(&rectangle1, ());

    // Line that entirely misses the rectangle
    let collision = rectangle1.line_collision(&(Coord2(0.0, 0.0), Coord2(0.0, 10.0)));

    assert!(collision.is_none());
}

#[test]
fn classify_simple_path() {
    // Create a rectangle
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let mut rectangle1 = GraphPath::from_path(&rectangle1, ());

    // Mark everything as an exterior path
    let first_edge = rectangle1.edges_for_point(0).nth(0).unwrap().into();
    rectangle1.classify_exterior_edges(first_edge, |_graph, _start_edge, _choices| panic!("Should be no intersections"));

    // All edges should be exterior
    for point_idx in 0..(rectangle1.num_points()) {
        let edges = rectangle1.edges_for_point(point_idx).collect::<Vec<_>>();

        assert!(edges.len() == 1);
        assert!(edges[0].kind() == GraphPathEdgeKind::Exterior);
    }
}

#[test]
fn get_path_from_exterior_lines() {
    // Create a rectangle
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let mut rectangle1 = GraphPath::from_path(&rectangle1, ());

    // Mark everything as an exterior path
    let first_edge = rectangle1.edges_for_point(0).nth(0).unwrap().into();
    rectangle1.classify_exterior_edges(first_edge, |_graph, _start_edge, _choices| panic!("Should be no intersections"));

    // Turn back into a path
    let rectangle2 = rectangle1.exterior_paths::<SimpleBezierPath>();

    println!("{:?}", rectangle2);

    assert!(rectangle2.len() == 1);
    assert!(rectangle2[0].start_point() == Coord2(1.0, 1.0));

    let points = rectangle2[0].points().collect::<Vec<_>>();
    assert!(points.len() == 4);

    assert!(points[0].2 == Coord2(1.0, 5.0));
    assert!(points[1].2 == Coord2(5.0, 5.0));
    assert!(points[2].2 == Coord2(5.0, 1.0));
    assert!(points[3].2 == Coord2(1.0, 1.0));
}

#[test]
fn get_path_from_exterior_lines_multiple_paths() {
    // Create a rectangle
    let rectangle1 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(1.0, 1.0))
        .line_to(Coord2(1.0, 5.0))
        .line_to(Coord2(5.0, 5.0))
        .line_to(Coord2(5.0, 1.0))
        .line_to(Coord2(1.0, 1.0))
        .build();
    let rectangle2 = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(11.0, 1.0))
        .line_to(Coord2(11.0, 5.0))
        .line_to(Coord2(15.0, 5.0))
        .line_to(Coord2(15.0, 1.0))
        .line_to(Coord2(11.0, 1.0))
        .build();
    let rectangle1      = GraphPath::from_path(&rectangle1, ());
    let rectangle2      = GraphPath::from_path(&rectangle2, ());
    let mut rectangle1  = rectangle1.merge(rectangle2);

    // Mark everything as an exterior path
    let first_edge = rectangle1.edges_for_point(0).nth(0).unwrap().into();
    rectangle1.classify_exterior_edges(first_edge, |_graph, _start_edge, _choices| panic!("Should be no intersections"));

    let first_edge = rectangle1.edges_for_point(4).nth(0).unwrap().into();
    rectangle1.classify_exterior_edges(first_edge, |_graph, _start_edge, _choices| panic!("Should be no intersections"));

    // Turn back into a path
    let rectangle3 = rectangle1.exterior_paths::<SimpleBezierPath>();

    println!("{:?}", rectangle3);

    assert!(rectangle3.len() == 2);
    assert!(rectangle3[0].start_point() == Coord2(1.0, 1.0));
    assert!(rectangle3[1].start_point() == Coord2(11.0, 1.0));

    let points = rectangle3[0].points().collect::<Vec<_>>();
    assert!(points.len() == 4);

    assert!(points[0].2 == Coord2(1.0, 5.0));
    assert!(points[1].2 == Coord2(5.0, 5.0));
    assert!(points[2].2 == Coord2(5.0, 1.0));
    assert!(points[3].2 == Coord2(1.0, 1.0));

    let points = rectangle3[1].points().collect::<Vec<_>>();
    assert!(points.len() == 4);

    assert!(points[0].2 == Coord2(11.0, 5.0));
    assert!(points[1].2 == Coord2(15.0, 5.0));
    assert!(points[2].2 == Coord2(15.0, 1.0));
    assert!(points[3].2 == Coord2(11.0, 1.0));
}

#[test]
fn collide_circles() {
    // Two overlapping circles
    let circle1 = Circle::new(Coord2(5.0, 5.0), 4.0).to_path::<SimpleBezierPath>();
    let circle2 = Circle::new(Coord2(12.9, 5.0), 4.0).to_path::<SimpleBezierPath>();

    // Create a graph path from the first one
    let graph_path = GraphPath::from_path(&circle1, 1);
    let graph_path = graph_path.collide(GraphPath::from_path(&circle2, 2), 0.01);

    // There are four points in each circle and there should be two collision points for 10 points total
    assert!(graph_path.num_points() == 10);

    // Display the graph
    for point_idx in 0..10 {
        println!("Point {:?}", point_idx);
        for edge in graph_path.edges_for_point(point_idx) {
            println!("  {:?} -> {:?} ({:?})", edge.start_point(), edge.end_point(), edge.end_point_index());
        }
    }

    // First four points should correspond to the four points in circle1 (and should all have one edge)
    // Some implementation details depended on here: 
    //   * we preserve at least the points from the first path when colliding
    assert!(graph_path.edges_for_point(0).collect::<Vec<_>>().len() == 1);
    assert!(graph_path.edges_for_point(1).collect::<Vec<_>>().len() == 1);
    assert!(graph_path.edges_for_point(2).collect::<Vec<_>>().len() == 1);
    assert!(graph_path.edges_for_point(3).collect::<Vec<_>>().len() == 1);

    // Point 1 should lead to the intersection point
    let to_intersection     = graph_path.edges_for_point(0).nth(0).unwrap();
    let intersection_point  = to_intersection.end_point_index();

    assert!(intersection_point > 3);

    // Intersection point should lead to another intersection point
    let intersection_edges = graph_path.edges_for_point(intersection_point).collect::<Vec<_>>();
    assert!(intersection_edges.len() == 2);

    // Should lead to one point in the second circle, and one other intersection point
    let is_intersection = |point_num| { graph_path.edges_for_point(point_num).collect::<Vec<_>>().len() > 1 };

    assert!(intersection_edges.iter().any(|edge| !is_intersection(edge.end_point_index())));
    assert!(intersection_edges.iter().any(|edge| is_intersection(edge.end_point_index())));

    // The following intersection point should have one point that leads back into our path
    let following_intersection      = intersection_edges.iter().filter(|edge| is_intersection(edge.end_point_index())).nth(0).unwrap();
    let second_intersection_edges   = graph_path.edges_for_point(following_intersection.end_point_index()).collect::<Vec<_>>();

    assert!(second_intersection_edges.iter().any(|edge| edge.end_point_index() <= 3));

    // It should also have a point that leads back to the first intersection, forming a loop
    assert!(second_intersection_edges.iter().any(|edge| edge.end_point_index() == intersection_point));
}
