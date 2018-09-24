use super::curve_line::*;
use super::super::curve::*;
use super::super::super::line::*;
use super::super::super::coordinate::*;

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
/// Determines where two curves intersect
/// 
/// This can return one of three possible values: a found intersection, an indication that the curves
/// should be subdivided and checked again or an indication that the curves do not intersect
/// 
fn curve_intersection_inner<'a, C: BezierCurve>(curve1: &'a C, curve2: &'a C, accuracy_area: f64) -> CurveIntersection
where C::Point: 'a+Coordinate2D {
    // TODO: we can calculate if curve1 or curve2 is approximately linear and switch to the line intersection algorithm

    // The bounds formed by the control points is faster to calculate than the exact curve bounds and good enough for our purposes
    let bounds1 = curve1.fast_bounding_box::<Bounds<_>>();
    let bounds2 = curve2.fast_bounding_box::<Bounds<_>>();

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
/// Determines the points where two curves intersect (using an approximation and subdividing using the bounding-box)
/// 
/// The accuracy level determines the smallest bounding box used before we estimate an intersection
/// 
pub fn curve_intersects_curve_bbox<'a, C: BezierCurve>(curve1: &'a C, curve2: &'a C, accuracy: f64) -> Vec<(f64, f64)>
where C::Point: 'a+Coordinate2D {
    // TODO: curves that are the same will produce far too many points of overlap
    // TODO: as will curves that have sections that are the same
    // TODO: repeatedly recalculating the t values as the recursion unwinds is inefficient

    // Calculate the accuracy area (we consider that we've got a match if we shrink one or both curves bounding boxes to this size)
    let accuracy_area = accuracy*accuracy;

    // Subdivide both curves
    let (curve1a, curve1b) = curve1.subdivide::<Curve<_>>(0.5);
    let (curve2a, curve2b) = curve2.subdivide::<Curve<_>>(0.5);

    // Curves needing to be subdivided after this intersection. This contains two pairs of (curve, offset) structures
    // offset is 0.5 if the curve if on the RHS, so we can do t*0.5+offset to get the t value to return before the
    // subdivision
    let mut to_subdivide = vec![];

    // Try intersecting them. If we get a match, stop and don't subdivide further (assume that if all our subdivisions generate a match that it's really the same one)
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

    match curve_intersection_inner(&curve1b, &curve2b, accuracy_area) {
        CurveIntersection::None             => (),
        CurveIntersection::Match(t1, t2)    => { return vec![(t1*0.5+0.5, t2*0.5+0.5)]; },
        CurveIntersection::Subdivide        => to_subdivide.push(((&curve1b, 0.5), (&curve2b, 0.5)))
    }

    // Search for matches in the curves to subdivide
    let mut result = vec![];
    for ((curve1, offset1), (curve2, offset2)) in to_subdivide.into_iter() {
        // Recursively search for more intersections
        let matches = curve_intersects_curve_bbox(curve1, curve2, accuracy);

        // If we find any, translate the 't' values to be in the range for our source curve
        // TODO: it'd be more efficient to do this in a single operation rather than at each recursion level
        for (t1, t2) in matches {
            result.push((t1*0.5+offset1, t2*0.5+offset2));
        }
    }

    result
}
