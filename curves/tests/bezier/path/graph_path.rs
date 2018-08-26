use flo_curves::*;
use flo_curves::bezier::*;
use flo_curves::bezier::path::*;

#[test]
pub fn create_and_read_simple_graph_path() {
    let path            = (Coord2(10.0, 11.0), vec![(Coord2(15.0, 16.0), Coord2(17.0, 18.0), Coord2(19.0, 20.0)), (Coord2(21.0, 22.0), Coord2(23.0, 24.0), Coord2(25.0, 26.0))]);
    let graph_path      = GraphPath::from_path(&path);

    assert!(graph_path.num_points() == 3);

    // Point 0 edges
    {
        let edges = graph_path.edges(0).collect::<Vec<_>>();

        assert!(edges.len() == 1);
        assert!(edges[0].kind() == GraphPathEdgeKind::Exterior);
        assert!(edges[0].start_point() == Coord2(10.0, 11.0));
        assert!(edges[0].end_point() == Coord2(19.0, 20.0));
        assert!(edges[0].control_points() == (Coord2(15.0, 16.0), Coord2(17.0, 18.0)));
    }

    // Point 1 edges
    {
        let edges = graph_path.edges(1).collect::<Vec<_>>();
        assert!(edges.len() == 1);
        assert!(edges[0].kind() == GraphPathEdgeKind::Exterior);
        assert!(edges[0].start_point() == Coord2(19.0, 20.0));
        assert!(edges[0].end_point() == Coord2(25.0, 26.0));
        assert!(edges[0].control_points() == (Coord2(21.0, 22.0), Coord2(23.0, 24.0)));
    }

    // Point 2 edges
    {
        let edges = graph_path.edges(2).collect::<Vec<_>>();
        assert!(edges.len() == 1);
        assert!(edges[0].kind() == GraphPathEdgeKind::Exterior);
        assert!(edges[0].start_point() == Coord2(19.0, 20.0));
        assert!(edges[0].end_point() == Coord2(10.0, 11.0));
    }
}
