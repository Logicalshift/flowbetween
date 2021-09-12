use crate::region::*;

use flo_curves::*;
use flo_curves::bezier::*;

use std::sync::*;
use std::time::{Duration};

const TOLERANCE: f64 = 0.01;

///
/// A point in a motion path
///
#[derive(Clone)]
struct MotionPoint {
    cp1:            Coord2,
    cp2:            Coord2,
    end_point:      Coord2,
    segment_length: f64
}

///
/// Effect that moves a region across a bezier path at linear speed
///
#[derive(Clone)]
pub struct MotionEffect {
    /// The initial coordinate of the path
    start_point: Coord2,

    /// The coordinates of the segments of the path that the animation should move along
    path: Vec<MotionPoint>,

    /// The total length of the path
    total_length: f64,

    /// The duration taken to move through the path
    duration: f64
}

impl MotionEffect {
    ///
    /// Creates a motion effect along the specified path
    ///
    pub fn from_points(duration: Duration, start_point: Coord2, path: Vec<(Coord2, Coord2, Coord2)>) -> MotionEffect {
        // Measure the distance of each path to find the total length
        let mut motion_path     = vec![];
        let mut last_point      = start_point;
        let mut total_length    = 0.0;

        for (cp1, cp2, end_point) in path {
            let path_section    = Curve::from_points(last_point, (cp1, cp2), end_point);
            let length          = curve_length(&path_section, TOLERANCE);

            motion_path.push(MotionPoint {
                cp1:            cp1,
                cp2:            cp2,
                end_point:      end_point,
                segment_length: length
            });

            total_length        += length;

            last_point          = end_point;
        }

        MotionEffect {
            start_point:    start_point,
            path:           motion_path,
            total_length:   total_length,
            duration:       (duration.as_nanos() as f64) / 1_000_000.0
        }
    }

    ///
    /// Returns the offset from the start point at the specified time
    ///
    pub fn offset_at_time(&self, time: f64, tolerance: f64) -> Coord2 {
        // Work out the distance down the path that we want for this time
        let distance                = time / self.duration * self.total_length;

        // Find the path segment that this distance appears along
        let mut start_point         = self.start_point;
        let mut segment_iter        = self.path.iter();
        let mut segment_distance    = distance;
        let segment;
        let segment_length;

        loop {
            if let Some(point) = segment_iter.next() {
                if point.segment_length >= segment_distance {
                    // The motion point lies along this segment
                    segment         = Curve::from_points(start_point, (point.cp1, point.cp2), point.end_point);
                    segment_length  = point.segment_length;
                    break;
                }

                // Move to the start of the next egment
                start_point         = point.end_point;
                segment_distance    -= point.segment_length;
            } else {
                // No point matches this distance (the end of the path is where everything appears after this pont)
                return start_point;
            }
        }

        // Estimate a t location for this distance
        let mut min_t       = 0.0;
        let mut t           = segment_distance / segment_length;

        // Search the end part of the curve if t is below the required length
        let mut t_length    = curve_length(&segment.section(0.0, t), TOLERANCE);
        let mut closest_t   = t;

        if t_length < segment_distance {
            min_t   = t;
            t       = 1.0;
        }

        // Search for a t value within tolerance of the target position
        let tolerance_sq    = tolerance * tolerance;
        while (segment_distance - t_length) * (segment_distance - t_length) > tolerance_sq {
            // Pick a value between min_t and t and measure it
            closest_t   = (min_t + t) * 0.5;
            t_length    = curve_length(&segment.section(0.0, closest_t), TOLERANCE);

            // Move min_t or t depending on which is closer to our target distance
            if t_length < segment_distance {
                min_t = closest_t;
            } else {
                t = closest_t;
            }
        }

        // Offset is the point on the curve at t minus the start point
        let curve_point = segment.point_at_pos(closest_t);
        let offset      = curve_point - self.start_point;

        offset
    }
}

impl AnimationEffect for MotionEffect {
    ///
    /// Returns the duration of this effect (or None if this effect will animate forever)
    ///
    /// If the effect is passed a time that's after where the 'duration' has completed it should always generate the same result
    ///
    fn duration(&self) -> Option<f64> {
        Some(self.duration)
    }

    ///
    /// Given the contents of the regions for this effect, calculates the path that should be rendered
    ///
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: Duration) -> Arc<AnimationRegionContent> {
        // Get the offset for the region contents
        let time    = (time.as_nanos() as f64) / 1_000_000.0;
        let offset  = self.offset_at_time(time, 0.01);

        // Move all of the paths in the region by the offset
        let paths   = region_contents.paths()
            .map(|path| path.offset_by(offset));

        Arc::new(AnimationRegionContent::from_paths(paths))
    }
}
