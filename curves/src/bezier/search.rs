use super::bounds::*;
use super::subdivide::*;
use super::super::coordinate::*;

///
/// Performs a subdivision search on a curve for a point matching a function
/// 
/// This searches for a point using a matching function that determines whether or not the point
/// is within a particular bounding box. The return value is a list of t values for the curve
/// described by the w values where the bounding box was shrunk to the size specified by min_size.
/// 
/// A limitation of this algorithm is that if the target point lies very close to a subdivision point,
/// it may produce multiple matches (as it will find a nearby point on either side of the subdivision)
/// 
pub fn search_bounds4<Point, MatchFn>(min_size: f64, w1: Point, w2: Point, w3: Point, w4: Point, match_fn: MatchFn) -> Vec<f64> 
where   Point:      Coordinate,
        MatchFn:    Fn(Point, Point) -> bool {
    // Helper function to determine if a bounding box is below the minimum size
    let min_size_squared    = min_size * min_size;
    let is_valid_match      = |p1: Point, p2: Point| {
        let diff            = p1-p2;
        let size_squared    = diff.dot(&diff);

        size_squared <= min_size_squared
    };

    // Push the initial curve as one to check
    let mut pending = vec![];
    let mut result  = vec![];

    // Each point is the list of w values and the min/max t values remaining to search
    pending.push((w1, w2, w3, w4, 0.0, 1.0));

    // Iterate while there are still curve sections to search
    while let Some((w1, w2, w3, w4, min_t, max_t)) = pending.pop() {
        // Subdivide at the midpoint
        let midpoint = (min_t + max_t)/2.0;
        let ((aw1, aw2, aw3, aw4), (bw1, bw2, bw3, bw4)) = subdivide4(0.5, w1, w2, w3, w4);
        
        // Compute the bounds of either side
        let (amin, amax) = bounding_box4(aw1, aw2, aw3, aw4);
        let (bmin, bmax) = bounding_box4(bw1, bw2, bw3, bw4);

        // Process the 'earlier' side of the curve
        if match_fn(amin, amax) {
            if is_valid_match(amin, amax) {
                // Bounds are small enough this counts as a match: push the midpoint
                result.push((min_t+midpoint)/2.0);
            } else {
                // Continue processing this half of the curve
                pending.push((aw1, aw2, aw3, aw4, min_t, midpoint));
            }
        }

        // Process the 'later' side of the curve
        if match_fn(bmin, bmax) {
            if is_valid_match(bmin, bmax) {
                // Bounds are small enough this counts as a match: push the midpoint
                result.push((midpoint+max_t)/2.0);
            } else {
                // Continue processing this half of the curve
                pending.push((bw1, bw2, bw3, bw4, midpoint, max_t));
            }
        }
    }

    result
}
