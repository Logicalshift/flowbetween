use super::super::curve::*;
use super::super::basis::*;
use super::super::super::line::*;
use super::super::super::coordinate::*;

use roots::{find_roots_cubic, Roots};

///
/// Find the t values where a curve intersects a ray
///
/// Return value is a vector of (curve_t, line_t, intersection_point) values. The `line_t` value can be outside the
/// original line, so this will return all the points on the curve that lie on a line of infinite length.
/// 
pub fn curve_intersects_ray<C: BezierCurve, L: Line<Point=C::Point>>(curve: &C, line: &L) -> Vec<(f64, f64, C::Point)>
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
    let roots       = match roots {
        Roots::No(_)    => vec![],
        Roots::One(r)   => r.to_vec(),
        Roots::Two(r)   => r.to_vec(),
        Roots::Three(r) => r.to_vec(),
        Roots::Four(r)  => r.to_vec()
    };

    roots.into_iter()
        .map(|t| {
            // Allow a small amount of 'slop' for items at the start/end as the root finding is not exact
            if t < 0.0 && t > -0.01 {
                let factor      = (a*a + b*b).sqrt();
                let (a, b, c)   = (a/factor, b/factor, c/factor);
                let start_point = curve.start_point();
                
                if (start_point.x()*a + start_point.y()*b + c).abs() < 0.00001 {
                    0.0 
                } else {
                    t
                }
            } else if t > 1.0 && t < 1.01 { 
                let factor      = (a*a + b*b).sqrt();
                let (a, b, c)   = (a/factor, b/factor, c/factor);
                let end_point   = curve.end_point();

                if (end_point.x()*a + end_point.y()*b + c).abs() < 0.00001 {
                    1.0
                } else {
                    t
                }
            } else { t }
        })
        .map(|t| {
            (t, de_casteljau4(t, w1, w2, w3, w4))
        })
        .map(|(t, pos)| {
            // Coordinates on the curve
            let x   = pos.x();
            let y   = pos.y();

            // Solve for the position on the line
            let s = if b.abs() > 0.01 {
                (x-p1.x())/(p2.x()-p1.x())
            } else {
                (y-p1.y())/(p2.y()-p1.y())
            };

            (t, s, pos)
        })
        .filter(|(t, _s, _pos)| {
            // Point must be within the bounds of the line and the curve
            (t >= &0.0 && t <= &1.0)
        })
        .collect()
}

///
/// Find the t values where a curve intersects a line
///
/// Return value is a vector of (curve_t, line_t, intersection_point) values
/// 
pub fn curve_intersects_line<C: BezierCurve, L: Line<Point=C::Point>>(curve: &C, line: &L) -> Vec<(f64, f64, C::Point)>
where C::Point: Coordinate2D {
    let mut ray_interections = curve_intersects_ray(curve, line);
    ray_interections.retain(|(_t, s, _pos)| s >= &0.0 && s <= &1.0);

    ray_interections
}
