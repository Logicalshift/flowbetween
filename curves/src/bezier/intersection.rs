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
/// Determines the points where two curves intersect (using an approximation)
/// 
/// The accuracy level determines the smallest bounding box used before we estimate an intersection
/// 
pub fn curve_intersects_curve<C: BezierCurve>(curve1: &C, curve2: &C, accuracy: f64) -> Vec<(f64, f64)>
where C::Point: Coordinate2D {
    // TODO: curves that are the same will produce far too many points of overlap
    // TODO: as will curves that have sections that are the same
    // TODO: repeatedly recalculating the t values as the recursion unwinds is inefficient

    // Curves do not intersect if their bounding boxes do not overlap
    let bounds1 = curve1.bounding_box::<Bounds<C::Point>>();
    let bounds2 = curve2.bounding_box::<Bounds<C::Point>>();

    if !bounds1.overlaps(&bounds2) {
        // No overlap: curves do not intersect
        vec![]
    } else {
        // Overlap: curves may intersect (subdivide to search for intersections)
        let accuracy    = accuracy * accuracy;

        let size1       = bounds1.max() - bounds1.min();
        let area1       = size1.dot(&size1);
        let size2       = bounds2.max() - bounds2.min();
        let area2       = size2.dot(&size2);

        if area1 <= accuracy && area2 <= accuracy {
            // Both curves are smaller than the required accuracy (assume they meet in the middle)
            vec![(0.5, 0.5)]

        } else if area1 <= accuracy {

            // Subdivide curve2 only
            let (curve2a, curve2b) = curve2.subdivide(0.5);

            // Find the intersections of both sides
            let left_intersections  = curve_intersects_curve(curve1, &curve2a, accuracy);
            let right_intersections = curve_intersects_curve(curve1, &curve2b, accuracy);

            // Adjust the result to the t values of our curve
            // TODO: it's inefficient to do this repeatedly, would be better to pass in a t range and calculate it once
            let mut res = vec![];
            for (t1, t2) in left_intersections.into_iter() {
                res.push((t1, t2*0.5));
            }
            for (t1, t2) in right_intersections.into_iter() {
                res.push((t1, t2*0.5+0.5));
            }

            res

        } else if area2 <= accuracy {

            // Subdivide curve1 only
            let (curve1a, curve1b) = curve1.subdivide(0.5);

            // Find the intersections of both sides
            let left_intersections  = curve_intersects_curve(&curve1a, curve2, accuracy);
            let right_intersections = curve_intersects_curve(&curve1b, curve2, accuracy);

            // Adjust the result to the t values of our curve
            // TODO: it's inefficient to do this repeatedly, would be better to pass in a t range and calculate it once
            let mut res = vec![];
            for (t1, t2) in left_intersections.into_iter() {
                res.push((t1*0.5, t2));
            }
            for (t1, t2) in right_intersections.into_iter() {
                res.push((t1*0.5+0.5, t2));
            }

            res

        } else {

            // Subdivide both curves
            let (curve1a, curve1b) = curve1.subdivide(0.5);
            let (curve2a, curve2b) = curve2.subdivide(0.5);

            // Find the intersections of both sides
            let left_intersections  = curve_intersects_curve(&curve1a, &curve2a, accuracy);
            let right_intersections = curve_intersects_curve(&curve1b, &curve2b, accuracy);

            // Adjust the result to the t values of our curve
            // TODO: it's inefficient to do this repeatedly, would be better to pass in a t range and calculate it once
            let mut res = vec![];
            for (t1, t2) in left_intersections.into_iter() {
                res.push((t1*0.5, t2*0.5));
            }
            for (t1, t2) in right_intersections.into_iter() {
                res.push((t1*0.5+0.5, t2*0.5+0.5));
            }

            res
            
        }
    }
}
