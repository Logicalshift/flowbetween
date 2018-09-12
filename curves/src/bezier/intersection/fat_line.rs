use super::super::super::line::*;
use super::super::super::coordinate::*;

///
/// A 'fat line' is a line with a width. It's used in bezier intersection algorithms,
/// in particular the clipping algorithm described by Sederberg and Nishita
/// 
pub struct FatLine<Line> {
    /// The thin line L'
    line: Line,

    /// The distance from the line to the upper part of the 'fat line'
    d_min: f64,

    /// The distance from the line to the lower part of the 'fat line'
    d_max: f64,

    /// The coefficients (a, b, c) in the equation ax+bx+c (where a^2+b^2 = 0)
    coeff: (f64, f64, f64)
}

impl<L: Line> FatLine<L>
where L::Point: Coordinate2D {
    ///
    /// Creates a new fat line
    /// 
    #[inline]
    pub fn new(line: L, d_min: f64, d_max: f64) -> FatLine<L> {
        let (from, to)  = line.points();
        let (a, b, c)   = line_coefficients_2d(&line);

        FatLine {
            line:   line,
            d_min:  d_min,
            d_max:  d_max,
            coeff:  (a, b, c)
        }
    }
}
