use super::region::*;
use super::region_id::*;

use flo_curves::bezier::path::*;

///
/// Collects a list of animation regions into a set of IDs and paths
///
pub fn collect_regions<'a, Region: 'a+AnimationRegion, RegionIter: IntoIterator<Item=&'a Region>>(regions: RegionIter, time: f64) -> Vec<(RegionId, Vec<SimpleBezierPath>)> {
    regions
        .into_iter()
        .enumerate()
        .map(|(region_idx, region)| {
            (RegionId(region_idx), region.region(time))
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::region::*;
    use crate::animation_path::*;

    use std::sync::*;

    #[derive(Clone)]
    pub struct TestRegion;

    impl AnimationEffect for TestRegion {
        fn animate(&self, _region_contents: Arc<AnimationRegionContent>, _time: f64) -> Vec<AnimationPath> {
            vec![]
        }
    }

    impl AnimationRegion for TestRegion {
        fn region(&self, _time: f64) -> Vec<SimpleBezierPath> {
            vec![]
        }
    }

    #[test]
    fn collect_boxed_regions() {
        let boxed_regions = vec![Box::new(TestRegion)];

        let result = collect_regions(&boxed_regions, 0.0);

        assert!(result[0].0 == RegionId(0));
    }
}
