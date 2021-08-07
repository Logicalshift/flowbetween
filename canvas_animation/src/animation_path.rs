use flo_canvas::*;
use flo_curves::bezier::path::*;

use std::sync::*;

///
/// Describes a path rendered as part of an animation
///
pub struct AnimationPath {
    /// The time in milliseconds from the start of the keyframe where this content appears
    pub appearance_time: f64,

    /// The time in milliseconds from the start of the keyframe where this content is removed
    pub disappearance_time: Option<f64>,

    /// The attributes describe how this path is rendered
    pub attributes: Arc<Vec<PathAttribute>>,

    /// The path that will be rendered by this animation
    pub path: SimpleBezierPath
}
