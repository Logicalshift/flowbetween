use flo_curves::*;
use flo_curves::arc::*;
use flo_curves::bezier::path::*;
use flo_canvas_animation::*;
use flo_canvas_animation::effects::*;

use std::time::{Duration};

#[test]
pub fn add_region_to_motion_effect() {
    // Apply a circle region to a motion effect
    let circle      = Circle::new(Coord2(100.0, 100.0), 50.0).to_path::<SimpleBezierPath>();

    let effect      = LinearMotionEffect::from_points(Duration::from_millis(10000), Coord2(20.0, 30.0), vec![(Coord2(20.0, 100.0), Coord2(200.0, 200.0), Coord2(300.0, 400.0))]);
    let with_region = effect.with_region(vec![circle]);

    // The initial position of the region should match the circle we passed in
    let circle      = Circle::new(Coord2(100.0, 100.0), 50.0).to_path::<SimpleBezierPath>();
    let initial_pos = with_region.region(Duration::from_millis(0));
    assert!(initial_pos == vec![circle]);
}
