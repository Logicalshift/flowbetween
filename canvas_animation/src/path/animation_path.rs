use crate::path::animation_path_attributes::*;

use flo_canvas::*;
use flo_curves::bezier::path::*;

use std::iter;
use std::sync::*;
use std::time::{Duration};

///
/// Describes a path rendered as part of an animation
///
#[derive(Clone)]
pub struct AnimationPath {
    /// The time from the start of the keyframe where this content appears
    pub appearance_time: Duration,

    /// The attributes describe how this path is rendered
    pub attributes: AnimationPathAttribute,

    /// The path that will be rendered by this animation
    pub path: Arc<Vec<SimpleBezierPath>>
}

#[inline]
fn offset_path((start_point, points): &SimpleBezierPath, offset: Coord2) -> SimpleBezierPath {
    (*start_point + offset,
        points.iter().map(|(cp1, cp2, p)| (*cp1 + offset, *cp2 + offset, *p + offset)).collect())
}

impl AnimationPath {
    ///
    /// Creates a new animation path from a list of path operations
    ///
    pub fn from_path_ops<'a, PathOpIter: IntoIterator<Item=&'a PathOp>>(path: PathOpIter, appearance_time: Duration, attributes: AnimationPathAttribute) -> AnimationPath {
        let mut paths           = vec![];
        let mut current_path    = None;
        let mut last_point      = Coord2(0.0, 0.0);

        // Process the pathops into SimpleBezierPaths
        for path_op in path {
            match path_op {
                PathOp::NewPath     => { },

                PathOp::Move(x, y)  => {
                    // Add the subpath to the result
                    if let Some(new_path) = current_path.take() {
                        paths.push(new_path);
                    }

                    // Start a new path at this point
                    last_point      = Coord2(*x as _, *y as _);
                    current_path    = Some((last_point, vec![]));
                }

                PathOp::Line(x, y) => {
                    // Line is a bezier curve with points 1/3rd of the way along
                    let end_point   = Coord2(*x as _, *y as _);
                    let diff        = end_point - last_point;
                    let cp1         = (diff * (1.0/3.0)) + last_point;
                    let cp2         = (diff * (2.0/3.0)) + last_point;

                    current_path    = if let Some((start_point, mut points)) = current_path {
                        points.push((cp1, cp2, end_point));
                        Some((start_point, points))
                    } else {
                        Some((end_point, vec![]))
                    };

                    last_point      = end_point;
                }

                PathOp::BezierCurve(((cp1x, cp1y), (cp2x, cp2y)), (x, y)) => {
                    let end_point   = Coord2(*x as _, *y as _);
                    let cp1         = Coord2(*cp1x as _, *cp1y as _);
                    let cp2         = Coord2(*cp2x as _, *cp2y as _);

                    current_path    = if let Some((start_point, mut points)) = current_path {
                        points.push((cp1, cp2, end_point));
                        Some((start_point, points))
                    } else {
                        Some((end_point, vec![]))
                    };

                    last_point      = end_point;
                }

                PathOp::ClosePath => {
                    // Line back to the initial point
                    current_path    = if let Some((start_point, mut points)) = current_path {
                        let end_point   = start_point;
                        let diff        = end_point - last_point;
                        let cp1         = (diff * (1.0/3.0)) + last_point;
                        let cp2         = (diff * (2.0/3.0)) + last_point;

                        points.push((cp1, cp2, end_point));
                        last_point      = end_point;

                        Some((start_point, points))
                    } else {
                        None
                    };
                }
            }
        }

        // Add the final subpath to the result
        if let Some(new_path) = current_path.take() {
            paths.push(new_path);
        }

        AnimationPath {
            appearance_time:    appearance_time,
            attributes:         attributes,
            path:               Arc::new(paths)

        }
    }

    ///
    /// Creates a copy of this path that is offset by the specified distance
    ///
    pub fn offset_by(&self, distance: Coord2) -> AnimationPath {
        let dx = distance.x() as f32;
        let dy = distance.y() as f32;

        // Move the path coordinates
        let offset_path = self.path.iter()
            .map(|path| offset_path(path, distance))
            .collect();

        // Move the texture or other attributes 
        let attributes = match self.attributes.clone() {
            AnimationPathAttribute::FillTexture(texture_id, (x1, y1), (x2, y2), None, winding_rule) => {
                AnimationPathAttribute::FillTexture(texture_id, (x1 + dx, y1 + dy), (x2 + dx, y2 + dy), None, winding_rule)
            },

            AnimationPathAttribute::FillTexture(texture_id, (x1, y1), (x2, y2), Some(transform), winding_rule) => {
                let transform = Transform2D::translate(dx, dy) * transform;
                AnimationPathAttribute::FillTexture(texture_id, (x1, y1), (x2, y2), Some(transform), winding_rule)
            },

            AnimationPathAttribute::FillGradient(gradient_id, (x1, y1), (x2, y2), None, winding_rule) => {
                AnimationPathAttribute::FillGradient(gradient_id, (x1 + dx, y1 + dy), (x2 + dx, y2 + dy), None, winding_rule)
            },

            AnimationPathAttribute::FillGradient(gradient_id, (x1, y1), (x2, y2), Some(transform), winding_rule) => {
                let transform = Transform2D::translate(dx, dy) * transform;
                AnimationPathAttribute::FillGradient(gradient_id, (x1 + dx, y1 + dy), (x2 + dx, y2 + dy), Some(transform), winding_rule)
            },

            other => other
        };

        // Pack into a new path object
        AnimationPath {
            appearance_time:    self.appearance_time,
            attributes:         attributes,
            path:               Arc::new(offset_path)
        }
    }

    ///
    /// Converts this path into a list of path ops
    ///
    pub fn to_path_ops<'a>(&'a self) -> impl 'a+Iterator<Item=PathOp> {
        self.path.iter()
            .flat_map(|(start_point, points)| {
                iter::once(PathOp::Move(start_point.x() as _, start_point.y() as _))
                    .chain(points.iter().map(|(cp1, cp2, p)| {
                        PathOp::BezierCurve(((cp1.x() as _, cp1.y() as _), (cp2.x() as _, cp2.y() as _)), (p.x() as _, p.y() as _))
                    }))
                    .chain(iter::once(PathOp::ClosePath))
            })
    }

    ///
    /// Creates a path with identical attributes but a new set of operations
    ///
    pub fn with_path(&self, new_path: Arc<Vec<SimpleBezierPath>>) -> AnimationPath {
        AnimationPath {
            appearance_time:    self.appearance_time,
            attributes:         self.attributes,
            path:               new_path
        }
    }
}
