use super::region::*;
use super::region_id::*;

use flo_curves::bezier::path::*;

use std::time::{Duration};

///
/// Collects a list of animation regions into a set of IDs and paths
///
pub fn collect_regions<'a, Region: 'a+AnimationRegion, RegionIter: IntoIterator<Item=&'a Region>>(regions: RegionIter, time: Duration) -> Vec<(RegionId, Vec<SimpleBezierPath>)> {
    regions
        .into_iter()
        .enumerate()
        .map(|(region_idx, region)| {
            (RegionId(region_idx), region.region(time))
        })
        .collect()
}
