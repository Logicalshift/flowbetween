use super::line::*;
use super::super::coordinate::*;

///
/// For a two-dimensional line, computes the coefficients of the line equation ax+by+c=0
/// (such that a^2+b^2 = 1)
/// 
/// This will return (0,0,0) for a line where the start and end point are the same.
/// 
pub fn line_coefficients_2d<P: Coordinate+Coordinate2D, L: Line<Point=P>>(line: &L) -> (f64, f64, f64) {
    // Compute the offset 
    let (from, to)  = line.points();
    let offset      = to - from;

    // Compute values for a, b, c
    let (a, b, c)   = if offset.x() == 0.0 && offset.y() == 0.0 {
        // This is a point rather than a line
        return (0.0, 0.0, 0.0);
    } else if offset.x().abs() > offset.y().abs() {
        // Derive a, b, c from y = ax+c
        let a = offset.y() / offset.x();
        let b = -1.0;
        let c = -(a*from.x() + b*from.y());

        if offset.x() < 0.0 {
            (-a, -b, -c)
        } else {
            (a, b, c)
        }
    } else {
        // Derive a, b, c from x = by+c
        let a = -1.0;
        let b = offset.x() / offset.y();
        let c = -(a*from.x() + b*from.y());

        if offset.y() < 0.0 {
            (-a, -b, -c)
        } else {
            (a, b, c)
        }
    };

    // Normalise so that a^2+b^2 = 1
    let factor      = (a*a + b*b).sqrt();
    let (a, b, c)   = (a/factor, b/factor, c/factor);

    (a, b, c)
}
