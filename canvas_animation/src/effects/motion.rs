use crate::region::*;
use crate::animation_path::*;

use flo_curves::*;
use flo_curves::bezier::*;

use std::sync::*;

const TOLERANCE: f64 = 0.1;

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
    pub fn from_points(duration: f64, start_point: Coord2, path: Vec<(Coord2, Coord2, Coord2)>) -> MotionEffect {
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
            duration:       duration
        }
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
    fn animate(&self, region_contents: Arc<AnimationRegionContent>, time: f64) -> Vec<AnimationPath> {
        vec![]
    }

}
