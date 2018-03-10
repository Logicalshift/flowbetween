use super::*;

use std::f32;

///
/// Represents a rectangle
/// 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Rect {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32
}

impl Rect {
    ///
    /// Creates a new rectangle
    /// 
    pub fn new(top_left: PathPoint, bottom_right: PathPoint) -> Rect {
        Rect {
            x1: top_left.position.0,
            y1: top_left.position.1,
            x2: bottom_right.position.0,
            y2: bottom_right.position.1
        }
    }

    ///
    /// Creates an empty rectangle
    /// 
    pub fn empty() -> Rect {
        Rect {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0
        }
    }

    ///
    /// Converts a rectangle into one where x2 and y2 are greater than x1 and y1
    /// 
    #[inline]
    pub fn normalize(self) -> Rect {
        Rect {
            x1: f32::min(self.x1, self.x2),
            y1: f32::min(self.y1, self.y2),
            x2: f32::max(self.x1, self.x2),
            y2: f32::max(self.y1, self.y2),
        }
    }

    ///
    /// True if this rectangle has no size
    /// 
    #[inline]
    pub fn is_zero_size(&self) -> bool {
        self.x1 == self.x2 && self.y1 == self.y2
    }

    ///
    /// Creates the union of two rectangles
    /// 
    #[inline]
    pub fn union(self, rhs: Rect) -> Rect {
        if self.is_zero_size() {
            self
        } else if rhs.is_zero_size() {
            rhs
        } else {
            Rect {
                x1: f32::min(self.x1, f32::min(self.x2, f32::min(rhs.x1, rhs.x2))),
                y1: f32::min(self.y1, f32::min(self.y2, f32::min(rhs.y1, rhs.y2))),
                x2: f32::max(self.x1, f32::max(self.x2, f32::max(rhs.x1, rhs.x2))),
                y2: f32::max(self.y1, f32::max(self.y2, f32::max(rhs.y1, rhs.y2))),
            }
        }
    }
}
