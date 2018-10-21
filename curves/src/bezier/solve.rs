use super::curve::*;
use super::super::consts::*;
use super::super::coordinate::*;

use roots::{find_roots_cubic, Roots};

const CLOSE_ENOUGH: f64 = SMALL_DISTANCE * 50.0;

///
/// Solves for t in a single dimension for a bezier curve (finds the point(s) where the basis
/// function evaluates to p)
/// 
pub fn solve_basis_for_t(w1: f64, w2: f64, w3: f64, w4: f64, p: f64) -> Vec<f64> {
    // Compute the coefficients for the cubic bezier function
    let d = w1-p;
    let c = 3.0*(w2-w1);
    let b = 3.0*(w3-w2)-c;
    let a = w4-w1-c-b;

    // Solve for p
    let roots = find_roots_cubic(a, b, c, d);
    let mut roots = match roots {
        Roots::No(_)    => vec![],
        Roots::One(r)   => r.to_vec(),
        Roots::Two(r)   => r.to_vec(),
        Roots::Three(r) => r.to_vec(),
        Roots::Four(r)  => r.to_vec()
    };

    // Clip to 0/1 for small ranges outside
    for mut root in roots.iter_mut() {
        if *root < 0.0 && *root > -0.001 { *root = 0.0 }
        if *root > 1.0 && *root < 1.001 { *root = 1.0 }
    }

    // Remove any roots outside the range of the function
    roots.retain(|r| r >= &0.0 && r <= &1.0);

    // Return the roots
    roots
}

///
/// Given a point that is close to or on the specified bezier curve, solves the 't' value that can
/// be used to retrieve it
///
pub fn solve_curve_for_t<C: BezierCurve>(curve: &C, point: &C::Point) -> Option<f64> {
    let close_enough_sq = CLOSE_ENOUGH * CLOSE_ENOUGH;

    let p1              = curve.start_point();
    let (p2, p3)        = curve.control_points();
    let p4              = curve.end_point();

    // Solve the basis function for each of the point's dimensions and pick the first that appears close enough (and within the range 0-1)
    for dimension in 0..(C::Point::len()) {
        // Solve for this dimension
        let (w1, w2, w3, w4)    = (p1.get(dimension), p2.get(dimension), p3.get(dimension), p4.get(dimension));
        let possible_t_values   = solve_basis_for_t(w1, w2, w3, w4, point.get(dimension));

        for possible_t in possible_t_values {
            // Ignore values outside the range of the curve
            if possible_t < -0.001 || possible_t > 1.001 {
                continue;
            }

            // If this is an accurate enough solution, return this as the t value
            let point_at_t  = curve.point_at_pos(possible_t);
            let offset      = point_at_t - *point;
            if offset.dot(&offset) <= close_enough_sq {
                return Some(possible_t);
            }
        }
    }
    
    // No solution: result is None
    None
}
