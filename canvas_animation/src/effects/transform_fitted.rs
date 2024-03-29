use crate::region::*;
use crate::description::*;

use flo_curves::*;
use flo_curves::bezier::*;

use std::sync::*;
use std::cmp::{Ordering};
use std::time::{Duration};

///
/// Motion effect that's generated by fitting to a curve
///
#[derive(Clone)]
pub struct FittedTransformEffect {
    /// The anchor point around which all of the transformations are made
    anchor_point: Point2D,

    /// The initial point in the motion described by this effect
    start_point: TimeTransformPoint,

    /// The curve for the motion described by this effect
    curve: Vec<TimeCurveTransformPoint>
}

impl FittedTransformEffect {
    ///
    /// Creates a transform effect that smoothly moves through each of the specified points in time
    ///
    pub fn by_fitting_transformation(anchor_point: Point2D, points: Vec<TimeTransformPoint>) -> Option<FittedTransformEffect> {
        // Sort the points by time
        let mut points = points;
        points.sort_by(|p1, p2| p1.1.partial_cmp(&p2.1).unwrap_or(Ordering::Equal));

        // Attempt to fit a curve to these points
        let curves = fit_curve::<TimeTransformCurve>(&points, 0.01)?;

        if curves.len() == 0 {
            // We found a fit but it had nothing in it
            None
        } else {
            // Create the transform effect from the fitted curves
            let start_point     = curves[0].start_point();
            let curve_sections  = curves.into_iter()
                .map(|TimeTransformCurve(_start_point, section)| section)
                .collect();

            Some(FittedTransformEffect {
                anchor_point:   anchor_point,
                start_point:    start_point,
                curve:          curve_sections
            })
        }
    }

    ///
    /// Finds the transformation to use at a particular point in time
    ///
    pub fn transform_at_time(&self, time: Duration) -> TransformWithAnchor {
        // Get the time in terms of the curve
        let time = TimeTransformPoint::f64_from_duration(time);

        if time < self.start_point.1 {
            // This time is before this animation starts
            TransformWithAnchor(self.anchor_point, self.start_point.into())
        } else {
            // Find the first curve section with an end point after the time
            let mut start_point = &self.start_point;
            let mut section     = &self.curve[0];

            for idx in 0..self.curve.len() {
                // Check the time of this section
                section = &self.curve[idx];
                if section.end_point().1 > time {
                    // Pick this section if it's the first to end after the specified time
                    break;
                }

                // End point of this section is the start point of the next section
                start_point = &section.2;
            }

            if section.end_point().1 <= time {
                // The time is after the region affected by this transformation
                TransformWithAnchor(self.anchor_point, section.end_point().into())
            } else {
                // Solve the bezier curve at this point
                let curve = TimeTransformCurve(*start_point, *section);

                TransformWithAnchor(self.anchor_point, curve.transform_for_time_f64(time))
            }
        }
    }
}

impl AnimationEffect for FittedTransformEffect {
    ///
    /// Returns the duration of this effect (or None if this effect will animate forever)
    ///
    /// If the effect is passed a time that's after where the 'duration' has completed it should always generate the same result
    ///
    fn duration(&self) -> Option<f64> {
        Some(self.curve[self.curve.len()-1].end_point().1)
    }

    ///
    /// Given the contents of the regions for this effect, calculates the path that should be rendered
    ///
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: Duration) -> Arc<AnimationRegionContent> {
        // Get the transform for the region contents
        let transform   = self.transform_at_time(time);
        let transform   = transform.into();

        // Move all of the paths in the region by the offset
        let paths   = region_contents.paths()
            .map(|path| path.transform_by(&transform));

        Arc::new(AnimationRegionContent::from_paths(paths))
    }

    ///
    /// Given an input region that will remain fixed throughout the time period, returns a function that
    /// will animate it. This can be used to speed up operations when some pre-processing is required for
    /// the region contents, but is not always available as the region itself might be changing over time
    /// (eg, if many effects are combined)
    ///
    fn animate_cached(&self, region_contents: Arc<AnimationRegionContent>) -> Box<dyn Send+Fn(Duration) -> Arc<AnimationRegionContent>> {
        let cached_effect = self.clone();

        Box::new(move |time| {
            // Get the transform for the region contents
            let transform   = cached_effect.transform_at_time(time);
            let transform   = transform.into();

            // Move all of the paths in the region by the offset
            let paths   = region_contents.paths()
                .map(|path| path.transform_by(&transform));

            Arc::new(AnimationRegionContent::from_paths(paths))
        })
    }
}
