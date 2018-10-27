use super::graph_path::*;
use super::super::curve::*;
use super::super::super::geo::*;
use super::super::super::coordinate::*;

use std::fmt::Write;

///
/// Writes out the graph path as an SVG string
///
pub fn graph_path_svg_string<P: Coordinate+Coordinate2D, Label: Copy>(path: &GraphPath<P, Label>) -> String {
    let mut result = String::new();

    let bounds      = path.all_edges().fold(Bounds::empty(), |a, b| a.union_bounds(b.bounding_box()));;
    let offset      = bounds.min();
    let scale       = 1000.0/(bounds.max() - bounds.min()).x();

    let mut index   = 0;

    for edge in path.all_edges() {
        let start_point = edge.start_point();
        let end_point   = edge.end_point();
        let (cp1, cp2)  = edge.control_points();

        write!(result, "<!-- {}: Curve::from_points(Coord2({}, {}), Coord2({}, {}), Coord2({}, {}), Coord2({}, {})) -->\n", 
            index, 
            start_point.x(), start_point.y(),
            end_point.x(), end_point.y(),
            cp1.x(), cp1.y(),
            cp2.x(), cp2.y());

        let start_point = (start_point - offset)*scale;
        let end_point   = (end_point - offset)*scale;
        let cp1         = (cp1 - offset)*scale;
        let cp2         = (cp2 - offset)*scale;

        let kind        = match edge.kind() {
            GraphPathEdgeKind::Uncategorised    => "yellow",
            GraphPathEdgeKind::Visited          => "red",
            GraphPathEdgeKind::Exterior         => "blue",
            GraphPathEdgeKind::Interior         => "green"
        };

        write!(result, "<path d=\"M {} {} C {} {}, {} {}, {} {}\" fill=\"transparent\" stroke-width=\"1\" stroke=\"{}\" />\n",
            start_point.x(), start_point.y(),
            cp1.x(), cp1.y(),
            cp2.x(), cp2.y(),
            end_point.x(), end_point.y(),
            kind);
        write!(result, "<circle cx=\"{}\" cy=\"{}\" r=\"1.0\" fill=\"transparent\" stroke=\"magenta\" />\n", end_point.x(), end_point.y());
        write!(result, "<text style=\"font-size: 8pt\" dx=\"{}\" dy=\"{}\">{} - {}</text>\n", start_point.x()+4.0, start_point.y()+8.0, edge.end_point_index(), index);

        index += 1;
    }

    result
}