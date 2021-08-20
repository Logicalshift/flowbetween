use crate::region::*;
use crate::animation_path::*;

use flo_curves::bezier::*;

use std::sync::*;

///
/// The time curve effect applies a bezier curve time effect to an existing animation
///
/// It can be used to define simple effects like ease-in, ease-out or more complex
/// effects over time
///
#[derive(Clone)]
pub struct TimeCurveEffect<TEffect: AnimationEffect> {
    /// The effect that the time curve is being applied to
    effect: TEffect,

    /// The control points for the time curve (a 1D bezier curve, in time coordinates). Time is linear after the last control point,
    /// and the start point is always 0. Control points should always be moving forward in time (though there are no restrictions
    /// between two control points)
    ///
    /// Order is control point 1, control point 2, end point
    curve_points: Vec<(f64, f64, f64)>
}

impl<TEffect: AnimationEffect> TimeCurveEffect<TEffect> {
    ///
    /// Creates a new time curve effect with the specified control points
    ///
    pub fn with_control_points(effect: TEffect, control_points: Vec<(f64, f64, f64)>) -> TimeCurveEffect<TEffect> {
        TimeCurveEffect {
            effect:         effect,
            curve_points:   control_points
        }
    }

    ///
    /// Works out where the specified time lies on the curve
    ///
    pub fn time_for_time(&self, time: f64) -> f64 {
        // Find the two curve points that this time is between (first where the control point is greater than the time)
        let mut start_point = 0.0;
        let     cp1;
        let     cp2;
        let     end_point;

        let mut curve_iter = self.curve_points.iter();
        loop {
            if let Some((test_cp1, test_cp2, test_end_point)) = curve_iter.next() {
                // If the end point is > time then this is the section containing the requested time
                if test_end_point > &time {
                    // This is the curve section containing the specified time region
                    cp1         = *test_cp1;
                    cp2         = *test_cp2;
                    end_point   = *test_end_point;

                    break;
                }

                // The end point of this section is the start of the next section
                start_point = *test_end_point;
            } else {
                // The time is beyond the end of the curve, so we just treat it linearly
                return time;
            }
        }

        // We have the curve section with the time: the t value is the ratio that 'time' is along the curve
        let t = (time-start_point) / (end_point-start_point);

        // Time can be calculated using the bezier algorithm
        de_casteljau4(t, start_point, cp1, cp2, end_point)
    }
}

impl<TEffect: AnimationEffect> AnimationEffect for TimeCurveEffect<TEffect> {
    ///
    /// Given the contents of the regions for this effect, calculates the path that should be rendered
    ///
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: f64) -> Vec<AnimationPath> {
        self.effect.animate(region_contents, self.time_for_time(time))
    }

    ///
    /// Given an input region that will remain fixed throughout the time period, returns a function that
    /// will animate it. This can be used to speed up operations when some pre-processing is required for
    /// the region contents, but is not always available as the region itself might be changing over time
    /// (eg, if many effects are combined)
    ///
    fn animate_cached<'a>(&'a self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn 'a+Fn(f64) -> Vec<AnimationPath>> {
        let cached_effect = self.effect.animate_cached(region_contents);

        Box::new(move |time| {
            cached_effect(self.time_for_time(time))
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone)]
    pub struct TestEffect;

    impl AnimationEffect for TestEffect {
        fn animate(&self, _region_contents: Arc<AnimationRegionContent>, _time: f64) -> Vec<AnimationPath> {
            vec![]
        }
    }

    #[test]
    pub fn linear_time_curve() {
        let time_curve = TimeCurveEffect::with_control_points(TestEffect, vec![(100.0, 200.0, 300.0)]);

        let p1 = time_curve.time_for_time(50.0);
        assert!((p1-50.0).abs() < 0.1);
        let p2 = time_curve.time_for_time(200.0);
        assert!((p2-200.0).abs() < 0.1);
        let p3 = time_curve.time_for_time(300.0);
        assert!((p3-300.0).abs() < 0.1);
        let p4 = time_curve.time_for_time(400.0);
        assert!((p4-400.0).abs() < 0.1);
    }

    #[test]
    pub fn simple_easing_time_curve() {
        let time_curve = TimeCurveEffect::with_control_points(TestEffect, vec![(0.0, 300.0, 300.0)]);

        for p in 1..400 {
            let p2 = p as f64;
            let p1 = p2 - 1.0;
            let p3 = p2 + 1.0;

            let t1 = time_curve.time_for_time(p1);
            let t2 = time_curve.time_for_time(p2);
            let t3 = time_curve.time_for_time(p3);

            if p < 150 {
                assert!((t2-t1) <= (t3-t2));
            } else if p > 150 && p < 300 {
                assert!((t2-t1) >= (t3-t2));
            } else if p > 300 {
                assert!(((t2-t1)-(t3-t2)).abs() < 0.1);
            }
        }
    }
}
