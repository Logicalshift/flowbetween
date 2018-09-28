use super::path::*;
use super::graph_path::*;
use super::super::super::geo::*;
use super::super::super::coordinate::*;

///
/// Generates the path formed by adding two sets of paths
/// 
/// The input vectors represent the external edges of the path to add (a single BezierPath cannot have any holes in it, so a set of them
/// effectively represents a path intended to be rendered with an even-odd winding rule)
///
pub fn path_add<Point, P1: BezierPath<Point=Point>, P2: BezierPath<Point=Point>, POut: BezierPath<Point=Point>>(path1: &Vec<P1>, path2: &Vec<P2>) -> Vec<POut>
where   Point: Coordinate+Coordinate2D {
    unimplemented!()
}
