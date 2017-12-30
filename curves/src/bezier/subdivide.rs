use super::basis::*;
use super::super::coordinate::*;

///
/// Subdivides a cubic bezier curve at a particular point, returning the weights of
/// the two component curves
/// 
pub fn subdivide4<Point: Coordinate>(t: f64, w1: Point, w2: Point, w3: Point, w4: Point) -> 
    ((Point, Point, Point, Point), (Point, Point, Point, Point)) {
    // Weights (from de casteljau)
    let wn1 = w1*(1.0-t) + w2*t;
    let wn2 = w2*(1.0-t) + w3*t;
    let wn3 = w3*(1.0-t) + w4*t;

    // Further refine the weights
    let wnn1 = wn1*(1.0-t) + wn2*t;
    let wnn2 = wn2*(1.0-t) + wn3*t;

    // Get the point at which the two curves join
    let p = de_casteljau2(t, wnn1, wnn2);

    // Curves are built from the weight calculations and the final points
    ((w1, wn1, wnn1, p), (p, wnn2, wn3, w4))
}
