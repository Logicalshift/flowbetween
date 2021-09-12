use flo_canvas_animation::*;
use flo_canvas_animation::effects::*;

use std::sync::*;
use std::time::{Duration};

#[derive(Clone)]
pub struct TestEffect;

impl AnimationEffect for TestEffect {
    fn animate(&self, _region_contents: Arc<AnimationRegionContent>, _time: Duration) -> Arc<AnimationRegionContent> {
        Arc::new(AnimationRegionContent::default())
    }
}

#[test]
pub fn linear_time_curve() {
    let time_curve = TimeCurveEffect::with_control_points(TestEffect, vec![(100.0, 200.0, 300.0)]);

    let p1 = time_curve.time_for_time(Duration::from_millis(50));
    let p1 = (p1.as_nanos() as f64) / 1_000_000.0;
    assert!((p1-50.0).abs() < 0.1);
    let p2 = time_curve.time_for_time(Duration::from_millis(200));
    let p2 = (p2.as_nanos() as f64) / 1_000_000.0;
    assert!((p2-200.0).abs() < 0.1);
    let p3 = time_curve.time_for_time(Duration::from_millis(300));
    let p3 = (p3.as_nanos() as f64) / 1_000_000.0;
    assert!((p3-300.0).abs() < 0.1);
    let p4 = time_curve.time_for_time(Duration::from_millis(400));
    let p4 = (p4.as_nanos() as f64) / 1_000_000.0;
    assert!((p4-400.0).abs() < 0.1);
}

#[test]
pub fn simple_easing_time_curve() {
    let time_curve = TimeCurveEffect::with_control_points(TestEffect, vec![(0.0, 300.0, 300.0)]);

    for p in 1..400 {
        let p2 = p;
        let p1 = p2 - 1;
        let p3 = p2 + 1;

        let t1 = time_curve.time_for_time(Duration::from_millis(p1));
        let t2 = time_curve.time_for_time(Duration::from_millis(p2));
        let t3 = time_curve.time_for_time(Duration::from_millis(p3));

        let t1 = (t1.as_nanos() as f64) / 1_000_000.0;
        let t2 = (t2.as_nanos() as f64) / 1_000_000.0;
        let t3 = (t3.as_nanos() as f64) / 1_000_000.0;

        if p < 150 {
            assert!((t2-t1) <= (t3-t2));
        } else if p > 150 && p < 300 {
            assert!((t2-t1) >= (t3-t2));
        } else if p > 300 {
            assert!(((t2-t1)-(t3-t2)).abs() < 0.1);
        }
    }
}
