use flo_curves::coordinate::*;
use flo_curves::bezier::path::*;

use std::fmt::Write;

pub fn svg_path_string<Path: BezierPath>(path: &Path) -> String 
where Path::Point: Coordinate2D {
    let mut svg = String::new();

    write!(&mut svg, "M {} {}", path.start_point().x(), path.start_point().y());
    for (cp1, cp2, end) in path.points() {
        write!(&mut svg, " C {} {}, {} {}, {} {}", cp1.x(), cp1.y(), cp2.x(), cp2.y(), end.x(), end.y());
    }

    svg
}
