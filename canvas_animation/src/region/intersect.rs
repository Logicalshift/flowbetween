use super::region_id::*;

use flo_curves::*;
use flo_curves::bezier::path::*;

use std::iter;

///
/// Computes the bounding box of a set of paths
///
#[inline]
fn get_bounds<P: BezierPath>(path: &Vec<P>) -> Bounds<P::Point> {
    path.iter()
        .map(|path| path.fast_bounding_box::<Bounds<_>>())
        .fold(Bounds::empty(), |a, b| a.union_bounds(b))
}

///
/// Detects where a set of regions overlap and returns a set of non-overlapping regions (with the places where the region IDs combine)
///
pub fn intersect_regions(overlapping_regions: Vec<(RegionId, Vec<SimpleBezierPath>)>, precision: f64) -> Vec<(Vec<RegionId>, Vec<SimpleBezierPath>)> {
    // Turn the regions into the output format of this function (along with their bounding boxes)
    let mut regions = overlapping_regions.into_iter()
        .filter(|(_region_id, path)| path.len() > 0)
        .map(|(region_id, path)| {
            let bounds = get_bounds(&path);

            (iter::once(region_id).collect::<Vec<_>>(), path, bounds)
        })
        .collect::<Vec<_>>();

    // Intersect a past region with a future region, adding new regions to the end
    let mut first_region = 0;

    while first_region < regions.len()-1 {
        // For the second region, we don't compare the regions cut from the first region (which are added at the end)
        let mut second_region   = first_region + 1;
        let last_region         = regions.len();

        while second_region < last_region {
            // Intersect this region
            let (first_region_id, first_path, first_bounds)     = &regions[first_region];
            let (second_region_id, second_path, second_bounds)  = &regions[second_region];

            if first_bounds.overlaps(second_bounds) {
                // The bounds overlap: try intersecting the regions
                let intersection = path_full_intersect::<_, _, SimpleBezierPath>(first_path, second_path, precision);

                // If there's an intersection, update the regions
                if intersection.intersecting_path.len() > 0 {
                    // The intersection consists of both regions
                    let first_region_id     = first_region_id.clone();
                    let second_region_id    = second_region_id.clone();
                    let intersect_region_id = first_region_id.iter().cloned().chain(second_region_id.iter().cloned()).collect();

                    // The intersecting region is new, and added to the end (we'll avoid comparing it to first_region again)
                    let intersection_bounds = get_bounds(&intersection.intersecting_path);
                    regions.push((intersect_region_id, intersection.intersecting_path, intersection_bounds));

                    // Update the first and second regions
                    let [new_first_path, new_second_path] = intersection.exterior_paths;

                    let new_first_bounds    = get_bounds(&new_first_path);
                    regions[first_region]   = (first_region_id, new_first_path, new_first_bounds);

                    let new_second_bounds   = get_bounds(&new_second_path);
                    regions[second_region]  = (second_region_id, new_second_path, new_second_bounds);
                }
            }

            // Move on to the next region
            second_region += 1;
        }

        // We've intersected this region against everything it could intersect: move on to the next region
        first_region += 1;
    }

    // Reformat the regions into the output format (strip out the bounds)
    regions.into_iter()
        .filter(|(_, path, _)| path.len() > 0)
        .map(|(region_id, path, _)| (region_id, path))
        .collect()
}
