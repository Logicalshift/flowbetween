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
    let roots       = match roots {
        Roots::No(_)    => vec![],
        Roots::One(r)   => r.to_vec(),
        Roots::Two(r)   => r.to_vec(),
        Roots::Three(r) => r.to_vec(),
        Roots::Four(r)  => r.to_vec()
    };

    roots.into_iter()
        .filter(|t| {
            // Coordinates on the curve
            let pos = basis(*t, w1, w2, w3, w4);
            let x   = pos.x();
            let y   = pos.y();

            // Solve for the position on the line
            let s = if b.abs() > 0.01 {
                (x-p1.x())/(p2.x()-p1.x())
            } else {
                (y-p1.y())/(p2.y()-p1.y())
            };

            // Point must be within the bounds of the line and the curve
            (t >= &0.0 && t <= &1.0) && (s >= 0.0 && s < 1.0)
        })
        .collect()
}

///
/// Possible results of a curve intersection test
/// 
enum CurveIntersection {
    /// Curves do not intersect
    None,

    /// Curves might intersect but need to be subdivided
    Subdivide,

    /// Found an intersection between the two curves
    Match(f64, f64)
}

///
/// Returns the approximate bounds of a curve
/// 
/// BezierCurve::bounding_box is more precise but takes much longer to compute
/// 
#[inline]
fn simple_bounds<C: BezierCurve>(curve: &C) -> Bounds<C::Point> {
    let start           = curve.start_point();
    let end             = curve.end_point();
    let control_points  = curve.control_points();

    Bounds::bounds_for_points(vec![ start, end, control_points.0, control_points.1 ])
}

///
/// Determines where two curves intersect
/// 
/// This can return one of three possible values: a found intersection, an indication that the curves
/// should be subdivided and checked again or an indication that the curves do not intersect
/// 
fn curve_intersection_inner<C: BezierCurve>(curve1: &C, curve2: &C, accuracy_area: f64) -> CurveIntersection
where C::Point: Coordinate2D {
    // The bounds formed by the control points is faster to calculate than the exact curve bounds and good enough for our purposes
    let bounds1 = simple_bounds(curve1);
    let bounds2 = simple_bounds(curve2);

    if bounds1.overlaps(&bounds2) {
        // Compute the areas covered by the two curves
        let diff1   = bounds1.max()-bounds1.min();
        let diff2   = bounds2.max()-bounds2.min();
        let area1   = diff1.dot(&diff1);
        let area2   = diff2.dot(&diff2);

        if area1 <= accuracy_area {
            // If the area of the first curve is below the accuracy threshold, use a simple line intersection to return the result
            // Curves tend to become lines as we get more accurate
            let curve1_t = 0.5;
            let curve2_t = curve_intersects_line(curve2, &(curve1.start_point(), curve1.end_point()));

            if curve2_t.len() > 0 {
                CurveIntersection::Match(curve1_t, curve2_t[0])
            } else {
                CurveIntersection::None
            }
        } else if area2 <= accuracy_area {
            // Same, except the second curve has hit the accuracy threshold
            let curve1_t = curve_intersects_line(curve1, &(curve2.start_point(), curve2.end_point()));
            let curve2_t = 0.5;

            if curve1_t.len() > 0 {
                CurveIntersection::Match(curve1_t[0], curve2_t)
            } else {
                CurveIntersection::None
            }
        } else {
            // Both bounding boxes are above the accuracy threshold: need to subdivide further
            CurveIntersection::Subdivide
        }
    } else {
        // Bounding boxes don't overlap
        CurveIntersection::None
    }
}

///
/// Determines the points where two curves intersect (using an approximation)
/// 
/// The accuracy level determines the smallest bounding box used before we estimate an intersection
/// 
pub fn curve_intersects_curve<C: BezierCurve>(curve1: &C, curve2: &C, accuracy: f64) -> Vec<(f64, f64)>
where C::Point: Coordinate2D {
    // TODO: curves that are the same will produce far too many points of overlap
    // TODO: as will curves that have sections that are the same
    // TODO: repeatedly recalculating the t values as the recursion unwinds is inefficient

    // Calculate the accuracy area (we consider that we've got a match if we shrink one or both curves bounding boxes to this size)
    let accuracy_area = accuracy*accuracy;

    // Subdivide both curves
    let (curve1a, curve1b) = curve1.subdivide(0.5);
    let (curve2a, curve2b) = curve2.subdivide(0.5);

    // Curves needing to be subdivided after this intersection. This contains two pairs of (curve, offset) structures
    // offset is 0.5 if the curve if on the RHS, so we can do t*0.5+offset to get the t value to return before the
    // subdivision
    let mut to_subdivide = vec![];

    // Try intersecting them. If we get a match, stop and don't subdivide further
    match curve_intersection_inner(&curve1a, &curve2a, accuracy_area) {
        CurveIntersection::None             => (),
        CurveIntersection::Match(t1, t2)    => { return vec![(t1*0.5, t2*0.5)]; },
        CurveIntersection::Subdivide        => to_subdivide.push(((&curve1a, 0.0), (&curve2a, 0.0)))
    }

    match curve_intersection_inner(&curve1a, &curve2b, accuracy_area) {
        CurveIntersection::None             => (),
        CurveIntersection::Match(t1, t2)    => { return vec![(t1*0.5, t2*0.5+0.5)]; },
        CurveIntersection::Subdivide        => to_subdivide.push(((&curve1a, 0.0), (&curve2b, 0.5)))
    }

    match curve_intersection_inner(&curve1b, &curve2a, accuracy_area) {
        CurveIntersection::None             => (),
        CurveIntersection::Match(t1, t2)    => { return vec![(t1*0.5+0.5, t2*0.5)]; },
        CurveIntersection::Subdivide        => to_subdivide.push(((&curve1b, 0.5), (&curve2a, 0.0)))
    }

    match curve_intersection_inner(&curve1a, &curve2b, accuracy_area) {
        CurveIntersection::None             => (),
        CurveIntersection::Match(t1, t2)    => { return vec![(t1*0.5+0.5, t2*0.5+0.5)]; },
        CurveIntersection::Subdivide        => to_subdivide.push(((&curve1a, 0.5), (&curve2b, 0.5)))
    }

    // Search for matches in the curves to subdivide
    let mut result = vec![];
    for ((curve1, offset1), (curve2, offset2)) in to_subdivide {
        // Recursively search for more intersections
        let matches = curve_intersects_curve(curve1, curve2, accuracy);

        // If we find any, translate the 't' values to be in the range for our source curve
        // TODO: it'd be more efficient to do this in a single operation rather than at each recursion level
        for (t1, t2) in matches {
            result.push((t1*0.5+offset1, t2*0.5+offset2));
        }
    }

    result
}
