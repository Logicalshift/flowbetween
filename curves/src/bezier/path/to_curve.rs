use super::path::*;
use super::super::curve::*;

use itertools::*;

///
/// Converts a path to a series of bezier curves
/// 
pub fn path_to_curves<Path: BezierPath, Curve: BezierCurve<Point=Path::Point>>(path: &Path) -> impl Iterator<Item=Curve> {
    let just_start_point    = vec![(path.start_point(), path.start_point(), path.start_point())].into_iter();
    let points              = path.points();

    just_start_point.chain(points)
        .tuple_windows()
        .map(|((_, _, start_point), (cp1, cp2, end_point))| {
            Curve::from_points(start_point, end_point, cp1, cp2)
        })
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::super::super::*;

    #[test]
    pub fn can_convert_path_to_bezier_curves() {
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
}