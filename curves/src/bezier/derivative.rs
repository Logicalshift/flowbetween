use super::super::coordinate::*;

///
/// Returns the 1st derivative of a cubic bezier curve
/// 
pub fn derivative4<Point: Coordinate>(w1: Point, w2: Point, w3: Point, w4: Point) -> (Point, Point, Point) {
    ((w2-w1)*3.0, (w3-w2)*3.0, (w4-w3)*3.0)
}

///
/// Returns the 1st derivative of a quadratic bezier curve (or the 2nd derivative of a cubic curve)
/// 
pub fn derivative3<Point: Coordinate>(wn1: Point, wn2: Point, wn3: Point) -> (Point, Point) {
    ((wn2-wn1)*2.0, (wn3-wn2)*2.0)
}

///
/// Returns the 3rd derivative of a cubic bezier curve (2nd of a quadratic)
/// 
pub fn derivative2<Point: Coordinate>(wnn1: Point, wnn2: Point) -> Point {
    wnn2-wnn1
}
