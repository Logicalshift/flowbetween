use super::super::path::*;
use super::super::is_clockwise::*;
use super::super::super::super::coordinate::*;

/// Source of a path in the graphpath
#[derive(Copy, Clone, PartialEq)]
pub enum PathSource {
    Path1,
    Path2
}

/// Target of a path in the graphpath
#[derive(Copy, Clone, PartialEq)]
pub enum PathDirection {
    Clockwise,
    Anticlockwise
}

impl<'a, P: BezierPath> From<&'a P> for PathDirection
where P::Point: Coordinate2D {
    #[inline]
    fn from(path: &'a P) -> PathDirection {
        if path.is_clockwise() {
            PathDirection::Clockwise
        } else {
            PathDirection::Anticlockwise
        }
    }
}

/// Label attached to a path used for arithmetic
#[derive(Clone, Copy)]
pub struct PathLabel(pub PathSource, pub PathDirection);
