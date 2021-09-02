use super::animation_path::*;
use crate::region::*;

use flo_curves::*;
use flo_curves::bezier::path::*;

use std::time::{Duration};

impl AnimationPath {
    ///
    /// Returns true if this path is within the specified path
    ///
    pub fn inside_path<P: BezierPath>(&self, path: &Vec<P>) -> bool
    where P::Point: Coordinate+Coordinate2D {
        false
    }

    ///
    /// Returns true if this path is within the specified region
    ///
    pub fn inside_region<R: AnimationRegion>(&self, region: &R, time: Duration) -> bool {
        self.inside_path(&region.region(time))
    }
}
