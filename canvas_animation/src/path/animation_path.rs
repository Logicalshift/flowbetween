use crate::path::animation_path_attributes::*;

use flo_canvas::*;
use flo_curves::bezier::path::*;

use std::sync::*;

///
/// Describes a path rendered as part of an animation
///
#[derive(Clone)]
pub struct AnimationPath {
    /// The time in milliseconds from the start of the keyframe where this content appears
    pub appearance_time: f64,

    /// The time in milliseconds from the start of the keyframe where this content is removed
    pub disappearance_time: Option<f64>,

    /// The attributes describe how this path is rendered
    pub attributes: Arc<Vec<AnimationPathAttribute>>,

    /// The path that will be rendered by this animation
    pub path: SimpleBezierPath
}

impl AnimationPath {
    ///
    /// Creates a copy of this path that is offset by the specified distance
    ///
    pub fn offset_by(&self, distance: Coord2) -> AnimationPath {
        let (start_point, points) = &self.path;

        let new_start_point = *start_point + distance;
        let new_points = points.iter()
            .map(|(cp1, cp2, end_point)| {
                (*cp1 + distance, *cp2 + distance, *end_point + distance)
            })
            .collect();

        AnimationPath {
            appearance_time:    self.appearance_time,
            disappearance_time: self.disappearance_time,
            attributes:         Arc::clone(&self.attributes),
            path:               (new_start_point, new_points)
        }
    }
}
