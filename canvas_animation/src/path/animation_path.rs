use crate::path::animation_path_attributes::*;

use flo_canvas::*;

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
    pub attributes: AnimationPathAttribute,

    /// The path that will be rendered by this animation
    pub path: Arc<Vec<PathOp>>
}

#[inline]
fn offset_path_op(op: &PathOp, distance: &Coord2) -> PathOp {
    let dx = distance.x() as f32;
    let dy = distance.y() as f32;

    match op {
        PathOp::NewPath                                             => PathOp::NewPath,
        PathOp::ClosePath                                           => PathOp::ClosePath,
        PathOp::Move(x, y)                                          => PathOp::Move(*x + dx, *y + dy),
        PathOp::Line(x, y)                                          => PathOp::Line(*x + dx, *y + dy),
        PathOp::BezierCurve(((cp1x, cp1y), (cp2x, cp2y)), (x, y))   => PathOp::BezierCurve(((*cp1x + dx, *cp1y + dy), (*cp2x + dx, *cp2y + dy)), (*x + dx, *y + dy)),
    }
}

impl AnimationPath {
    ///
    /// Creates a copy of this path that is offset by the specified distance
    ///
    pub fn offset_by(&self, distance: Coord2) -> AnimationPath {
        let offset_path = self.path.iter()
            .map(|path_op| offset_path_op(path_op, &distance))
            .collect();

        AnimationPath {
            appearance_time:    self.appearance_time,
            disappearance_time: self.disappearance_time,
            attributes:         self.attributes.clone(),
            path:               Arc::new(offset_path)
        }
    }
}
