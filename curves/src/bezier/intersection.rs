use super::curve::*;
use super::basis::*;
use super::super::line::*;
use super::super::coordinate::*;

use roots::{find_roots_cubic, Roots};

///
/// Find the t values where a curve intersects a line
/// 
pub fn curve_intersects_line<C: BezierCurve, L: Line<Point=C::Point>>(curve: &C, line: &L) -> Vec<f64>
where C::Point: Coordinate2D {
    // Based upon https://www.particleincell.com/2013/cubic-line-intersection/

    // Line coefficients
    let (p1, p2)    = line.points();
    let a           = p2.y()-p1.y();
    let b           = p1.x()-p2.x();
    let c           = p1.x()*(p1.y()-p2.y()) + p1.y()*(p2.x()-p1.x());

    // Bezier coefficients
    let (w2, w3)    = curve.control_points();
    let (w1, w4)    = (curve.start_point(), curve.end_point());
    let bx          = bezier_coefficients(0, &w1, &w2, &w3, &w4);
    let by          = bezier_coefficients(1, &w1, &w2, &w3, &w4);

    let p           = (
        a*bx.0+b*by.0,
        a*bx.1+b*by.1,
        a*bx.2+b*by.2,
        a*bx.3+b*by.3+c
    );

    let roots       = find_roots_cubic(p.0, p.1, p.2, p.3);
    println!("{:?}", roots);
    let roots       = match roots {
        Roots::No(_)    => vec![],
        Roots::One(r)   => r.to_vec(),
        Roots::Two(r)   => r.to_vec(),
        Roots::Three(r) => r.to_vec(),
        Roots::Four(r)  => r.to_vec()
    };

    roots.into_iter()
        .collect()
}
