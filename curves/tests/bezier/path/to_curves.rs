use flo_curves::*;
use flo_curves::bezier::*;
use flo_curves::bezier::path::*;

#[test]
pub fn convert_path_to_bezier_curves() {
    let path            = (Coord2(10.0, 11.0), vec![(Coord2(15.0, 16.0), Coord2(17.0, 18.0), Coord2(19.0, 20.0)), (Coord2(21.0, 22.0), Coord2(23.0, 24.0), Coord2(25.0, 26.0))]);
    let curve           = path_to_curves::<_, Curve>(&path);
    let curve: Vec<_>   = curve.collect();

    assert!(curve.len() == 2);
    assert!(curve[0] == Curve {
        start_point:    Coord2(10.0, 11.0),
        end_point:      Coord2(19.0, 20.0),
        control_points: (Coord2(15.0, 16.0), Coord2(17.0, 18.0))
    });
    assert!(curve[1] == Curve {
        start_point:    Coord2(19.0, 20.0),
        end_point:      Coord2(25.0, 26.0),
        control_points: (Coord2(21.0, 22.0), Coord2(23.0, 24.0))
    });
}

#[test]
pub fn no_points_means_no_curve() {
    let path            = (Coord2(10.0, 11.0), vec![]);
    let curve           = path_to_curves::<_, Curve>(&path);
    let curve: Vec<_>   = curve.collect();

    assert!(curve.len() == 0);
}
