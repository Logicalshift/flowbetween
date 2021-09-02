use flo_canvas_animation::*;
use flo_curves::*;
use flo_curves::arc::*;
use flo_curves::bezier::path::*;

use std::sync::*;
use std::time::{Duration};

#[derive(Clone)]
pub struct TestRegion;

impl AnimationEffect for TestRegion {
    fn animate(&self, _region_contents: Arc<AnimationRegionContent>, _time: Duration) -> Vec<AnimationPath> {
        vec![]
    }
}

impl AnimationRegion for TestRegion {
    fn region(&self, _time: Duration) -> Vec<SimpleBezierPath> {
        vec![]
    }
}

#[test]
fn collect_boxed_regions() {
    let boxed_regions = vec![Box::new(TestRegion)];

    let result = collect_regions(&boxed_regions, Duration::from_millis(0));

    assert!(result[0].0 == RegionId(0));
}

#[test]
fn non_intersecting_regions() {
    let path1           = vec![Circle::new(Coord2(5.0, 5.0), 5.0).to_path::<SimpleBezierPath>()];
    let path2           = vec![Circle::new(Coord2(15.0, 5.0), 5.0).to_path::<SimpleBezierPath>()];

    let intersections   = intersect_regions(vec![(RegionId(0), path1), (RegionId(1), path2)], 0.01);

    assert!(intersections.len() == 2);
    assert!(intersections[0].0  == vec![RegionId(0)]);
    assert!(intersections[1].0  == vec![RegionId(1)]);
}

#[test]
fn overlapping_regions() {
    let path1           = vec![Circle::new(Coord2(5.0, 5.0), 5.0).to_path::<SimpleBezierPath>()];
    let path2           = vec![Circle::new(Coord2(8.0, 5.0), 5.0).to_path::<SimpleBezierPath>()];

    let intersections   = intersect_regions(vec![(RegionId(0), path1), (RegionId(1), path2)], 0.01);

    assert!(intersections.len() == 3);
    assert!(intersections[0].0  == vec![RegionId(0)]);
    assert!(intersections[1].0  == vec![RegionId(1)]);
    assert!(intersections[2].0  == vec![RegionId(0), RegionId(1)]);
}


#[test]
fn multi_overlapping_regions() {
    let path1           = vec![Circle::new(Coord2(5.0, 5.0), 5.0).to_path::<SimpleBezierPath>()];
    let path2           = vec![Circle::new(Coord2(8.0, 5.0), 5.0).to_path::<SimpleBezierPath>()];
    let path3           = vec![Circle::new(Coord2(5.0, 8.0), 5.0).to_path::<SimpleBezierPath>()];

    let intersections   = intersect_regions(vec![(RegionId(0), path1), (RegionId(1), path2), (RegionId(2), path3)], 0.01);

    println!("{:?}", intersections.len());
    println!("{:?}", intersections.iter().map(|(regions, _)| regions.clone()).collect::<Vec<_>>());

    assert!(intersections.len() == 7);
}
